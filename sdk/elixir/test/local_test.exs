defmodule FluentCI.LocalTest do
  use ExUnit.Case, async: true
  alias FluentCI.Host, as: H
  alias FluentCI.Host.{Journal, Local, Result}
  import Bitwise

  @engine System.get_env(
            "FLUENTCI_ENGINE_BIN",
            Path.expand("../../../target/release/fluentci-engine", __DIR__)
          )
  if not File.regular?(@engine) do
    if System.get_env("FLUENTCI_REQUIRE_ENGINE") == "1", do: raise("release engine required")
    @moduletag skip: "build release engine or set FLUENTCI_ENGINE_BIN"
  end

  @moduletag timeout: 30_000
  setup do
    root =
      Path.join(
        System.tmp_dir!(),
        "fluentci local-#{System.pid()}-#{System.unique_integer([:positive, :monotonic])}"
      )

    File.mkdir!(root)
    File.chmod!(root, 0o700)
    on_exit(fn -> File.rm_rf!(root) end)

    env =
      Map.new(for {key, _} <- System.get_env(), String.starts_with?(key, "OTEL_"), do: {key, nil})

    {:ok,
     root: root, opts: [binary: @engine, run_root: Path.join(root, "runs"), cwd: root, env: env]}
  end

  defp start(plan, opts) do
    assert {:ok, handle} = Local.start(plan, opts)
    on_exit(fn -> Local.close(handle, 10_000) end)
    handle
  end

  defp wait_until(fun) do
    deadline = System.monotonic_time(:millisecond) + 5000
    wait_until(fun, deadline)
  end

  defp wait_until(fun, deadline) do
    if not fun.() do
      assert System.monotonic_time(:millisecond) < deadline, "condition did not become true"
      Process.sleep(10)
      wait_until(fun, deadline)
    end
  end

  defp written?(path) do
    case File.read(path) do
      {:ok, bytes} -> String.trim(bytes) != ""
      _ -> false
    end
  end

  defp stopped?(pid) do
    case System.cmd("/bin/ps", ["-p", pid, "-o", "stat="], stderr_to_stdout: true) do
      {_, code} when code != 0 -> true
      {state, _} -> String.starts_with?(String.trim(state), "Z")
    end
  end

  defp looping_plan do
    task =
      H.task("loop", "/bin/sh", ".")
      |> H.args([
        "-c",
        "printf live-marker; /bin/sleep 20 & echo $! > child.pid; echo $$ > parent.pid; wait"
      ])
      |> H.timeout_ms(15_000)

    H.plan([task])
  end

  defp remaining_events(handle, cursor) do
    assert {:ok, batch} = Local.events(handle, cursor, 2)

    case batch.events do
      [] -> []
      events -> events ++ remaining_events(handle, batch.next_cursor)
    end
  end

  test "typed events are readable live and resume after completion and close", %{
    root: root,
    opts: opts
  } do
    task =
      H.task("live", "/bin/sh", ".")
      |> H.args([
        "-c",
        "printf 'live-你好\\n'; while [ ! -f release ]; do /bin/sleep 0.02; done; printf done"
      ])
      |> H.timeout_ms(15_000)

    handle = start(H.plan([task]), opts)

    wait_until(fn ->
      {:ok, batch} = Local.events(handle)

      Enum.any?(batch.events, fn event ->
        match?({:log, %{text: "live-你好\n"}}, event.kind)
      end)
    end)

    assert {:error, %Local.Error{kind: :timeout}} = Local.await(handle, 1)
    assert {:ok, first} = Local.events(handle)
    File.write!(Path.join(root, "release"), "go")
    assert {:ok, _} = Local.await(handle)
    events = first.events ++ remaining_events(handle, first.next_cursor)
    assert Enum.map(events, & &1.sequence) == Enum.to_list(0..(length(events) - 1))
    assert hd(events).kind == {:run_started, 1}
    assert List.last(events).kind == {:run_finished, true}
    assert Enum.any?(events, &match?({:task_state, %{status: :succeeded}}, &1.kind))
    assert :ok = Local.close(handle)

    assert {:ok, %Journal.Batch{events: [%Journal.Event{sequence: 0}]}} =
             Local.events(handle, 0, 1)
  end

  test "owned runs resolve paths once, isolate files and return cache hits", %{
    root: root,
    opts: opts
  } do
    File.mkdir!(Path.join(root, "source"))
    File.write!(Path.join(root, "source/input"), "artifact")

    task =
      H.task("build", "/bin/sh", ".")
      |> H.args(["-c", "/bin/cat input > output; printf log"])
      |> H.cache(["input"], ["output"], "fixture", relocatable: true)
      |> H.timeout_ms(5000)

    plan =
      H.plan([task])
      |> H.workspace("source", ["input"])
      |> H.cache_dir("cache")
      |> H.resource_dir("leases")

    original_cwd = File.cwd!()

    handles =
      for outcome <- ["stored", "hit"] do
        handle = start(plan, opts)
        assert {:ok, %Local.Report{result: report, exit_status: 0}} = Local.await(handle)
        assert report.outcome == :succeeded
        assert hd(report.tasks).cache.outcome == outcome
        assert File.read!(hd(report.tasks).stdout) == "log"
        assert File.read!(Path.join(handle.run_directory, "workspace/output")) == "artifact"
        assert {:ok, %Local.Report{result: ^report}} = Local.await(handle)
        wrapper = Path.dirname(handle.run_directory)
        assert band(File.stat!(wrapper).mode, 0o777) == 0o700
        assert band(File.stat!(Path.join(wrapper, "plan.json")).mode, 0o777) == 0o600
        wire = JSON.decode!(File.read!(Path.join(wrapper, "plan.json")))
        assert wire["workspace"]["source"] == Path.join(root, "source")
        assert wire["cache_dir"] == Path.join(root, "cache")
        assert wire["resource_dir"] == Path.join(root, "leases")
        assert :ok = Local.close(handle)
        assert File.regular?(Path.join(handle.run_directory, "result.json"))
        assert {:error, %Local.Error{kind: :closed}} = Local.await(handle)
        handle
      end

    assert length(Enum.uniq_by(handles, & &1.run_directory)) == 2
    assert File.cwd!() == original_cwd
    refute File.exists?(Path.join(root, "source/output"))
  end

  test "concurrent runs keep cwd and environment child-local", %{root: root, opts: opts} do
    original_cwd = File.cwd!()
    original_env = System.get_env("FLUENTCI_LOCAL_VALUE")

    handles =
      for value <- ["first", "second"] do
        directory = Path.join(root, value)
        File.mkdir!(directory)

        task =
          H.task(value, "/bin/sh", ".")
          |> H.args([
            "-c",
            "echo ready > started; while [ ! -f release ]; do /bin/sleep 0.02; done; printf '%s' \"$FLUENTCI_LOCAL_VALUE\""
          ])
          |> H.env(%{"FLUENTCI_LOCAL_VALUE" => value})
          |> H.timeout_ms(5000)

        child_opts =
          opts
          |> Keyword.put(:cwd, directory)
          |> Keyword.update!(:env, &Map.put(&1, "FLUENTCI_LOCAL_VALUE", "engine-only"))

        {value, directory, start(H.plan([task]), child_opts)}
      end

    for {_, directory, _} <- handles,
        do: wait_until(fn -> written?(Path.join(directory, "started")) end)

    for {_, directory, _} <- handles, do: File.touch!(Path.join(directory, "release"))

    for {value, _, handle} <- handles do
      assert {:ok, %Local.Report{result: result}} = Local.await(handle)
      assert File.read!(hd(result.tasks).stdout) == value
    end

    assert File.cwd!() == original_cwd
    assert System.get_env("FLUENTCI_LOCAL_VALUE") == original_env
  end

  test "cancellation waits for descendant cleanup and can be awaited repeatedly", %{
    root: root,
    opts: opts
  } do
    handle = start(looping_plan(), opts)
    wait_until(fn -> written?(Path.join(root, "parent.pid")) end)

    wait_until(fn ->
      case Local.output(handle) do
        {:ok, bytes} -> String.contains?(bytes, "live-marker")
        _ -> false
      end
    end)

    assert {:error, %Local.Error{kind: :timeout}} = Local.await(handle, 1)

    assert {:ok, %Local.Report{result: %Result{outcome: :cancelled}} = result} =
             Local.cancel(handle)

    assert {:ok, ^result} = Local.await(handle)
    assert {:ok, ^result} = Local.cancel(handle)

    for file <- ["parent.pid", "child.pid"] do
      pid = File.read!(Path.join(root, file)) |> String.trim()
      wait_until(fn -> stopped?(pid) end)
    end
  end

  test "owner death cancels the engine and releases the handle after cleanup", %{
    root: root,
    opts: opts
  } do
    parent = self()

    owner =
      spawn(fn ->
        {:ok, handle} = Local.start(looping_plan(), opts)
        send(parent, {:handle, handle})

        receive do
          :done -> :ok
        end
      end)

    on_exit(fn -> Process.exit(owner, :kill) end)
    assert_receive {:handle, handle}, 5000
    on_exit(fn -> Local.close(handle, 10_000) end)
    monitor = Process.monitor(handle.pid)
    wait_until(fn -> written?(Path.join(root, "parent.pid")) end)
    Process.exit(owner, :kill)
    assert_receive {:DOWN, ^monitor, :process, _, :normal}, 10_000

    assert {:ok, %Result{outcome: :cancelled}} =
             File.read!(Path.join(handle.run_directory, "result.json")) |> Result.decode()

    for file <- ["parent.pid", "child.pid"] do
      pid = File.read!(Path.join(root, file)) |> String.trim()
      wait_until(fn -> stopped?(pid) end)
    end
  end

  test "close and OTP stop both wait for active engine cleanup", %{root: root, opts: opts} do
    for operation <- [:close, :otp_stop, :supervisor_stop] do
      directory = Path.join(root, Atom.to_string(operation))
      File.mkdir!(directory)
      handle = start(looping_plan(), Keyword.put(opts, :cwd, directory))
      wait_until(fn -> written?(Path.join(directory, "parent.pid")) end)

      case operation do
        :close ->
          assert :ok = Local.close(handle)

        :otp_stop ->
          assert :ok = GenServer.stop(handle.pid, :normal, :infinity)

        :supervisor_stop ->
          assert :ok =
                   DynamicSupervisor.terminate_child(FluentCI.Host.Local.Supervisor, handle.pid)
      end

      assert {:ok, %Result{outcome: :cancelled}} =
               File.read!(Path.join(handle.run_directory, "result.json")) |> Result.decode()

      pid = File.read!(Path.join(directory, "child.pid")) |> String.trim()
      wait_until(fn -> stopped?(pid) end)
    end
  end

  test "pipeline failure returns a typed report rather than a transport failure", %{opts: opts} do
    task =
      H.task("fail", "/bin/sh", ".")
      |> H.args(["-c", "printf 'elixir process error' >&2; exit 7"])

    handle = start(H.plan([task]), opts)

    assert {:ok, %Local.Report{exit_status: 7, result: %Result{outcome: :failed, tasks: [task]}}} =
             Local.await(handle)

    assert task.exit_code == 7
    assert task.error =~ "exit code 7"
    assert task.error =~ "elixir process error"
  end

  test "invalid plans and configuration have no run-directory side effects", %{
    root: root,
    opts: opts
  } do
    task = H.task("duplicate", "/usr/bin/true", ".")
    assert {:error, %Local.Error{kind: :invalid_plan}} = Local.start(H.plan([task, task]), opts)

    assert {:error, %Local.Error{kind: :configuration}} =
             Local.start(H.plan(), Keyword.put(opts, :binary, Path.join(root, "missing")))

    assert {:error, %Local.Error{kind: :configuration}} =
             Local.start(H.plan(), Keyword.put(opts, :env, %{"BAD=KEY" => "secret"}))

    refute File.exists?(Path.join(root, "runs"))
    invalid = Path.join(root, "non-executable")
    File.write!(invalid, "not executable")
    File.chmod!(invalid, 0o600)

    assert {:error, %Local.Error{kind: :spawn}} =
             Local.start(H.plan(), Keyword.put(opts, :binary, invalid))

    assert File.ls!(Path.join(root, "runs")) == []
  end

  defp fake_engine(root, script) do
    path = Path.join(root, "fixture-engine")
    File.write!(path, "#!/bin/sh\n" <> script)
    File.chmod!(path, 0o700)
    path
  end

  test "output polling retains a bounded tail after process exit", %{root: root, opts: opts} do
    binary =
      fake_engine(
        root,
        "mkdir \"$4\"; printf '%070000d' 0; printf tail-marker; printf '{\"outcome\":\"succeeded\",\"tasks\":[]}' > \"$4/result.json\""
      )

    handle = start(H.plan(), Keyword.put(opts, :binary, binary))
    assert {:ok, %Local.Report{exit_status: 0}} = Local.await(handle)
    assert {:ok, bytes} = Local.output(handle)
    assert byte_size(bytes) == 65_536
    assert String.ends_with?(bytes, "tail-marker")
  end

  test "missing, malformed, oversized and nonregular reports return structured errors", %{
    root: root,
    opts: opts
  } do
    for {name, script, expected} <- [
          {"missing", "exit 3", :io},
          {"malformed", "mkdir \"$4\"; printf '{' > \"$4/result.json\"", :decode},
          {"large", "mkdir \"$4\"; printf '%017000000d' 0 > \"$4/result.json\"", :decode},
          {"fifo", "mkdir \"$4\"; /usr/bin/mkfifo \"$4/result.json\"", :io}
        ] do
      directory = Path.join(root, name)
      File.mkdir!(directory)
      binary = fake_engine(directory, script)
      handle = start(H.plan(), Keyword.put(opts, :binary, binary))
      assert {:error, %Local.Error{kind: ^expected, run_directory: path}} = Local.await(handle)
      assert path == handle.run_directory
    end
  end

  test "early cancellation either yields a terminal report or a startup error without live children",
       %{root: root, opts: opts} do
    handle = start(looping_plan(), opts)

    case Local.cancel(handle, 10_000) do
      {:ok, %Local.Report{result: %Result{outcome: outcome}}} ->
        assert outcome in [:cancelled, :succeeded]

      {:error, %Local.Error{kind: :io, exit_status: status}} ->
        assert status != 0
    end

    for file <- ["parent.pid", "child.pid"], File.exists?(Path.join(root, file)) do
      pid = File.read!(Path.join(root, file)) |> String.trim()
      wait_until(fn -> stopped?(pid) end)
    end
  end

  test "a valid report must still match the submitted task IDs", %{root: root, opts: opts} do
    report =
      JSON.encode!(%{
        "outcome" => "succeeded",
        "tasks" => [
          %{
            "id" => "wrong",
            "status" => "succeeded",
            "duration_ms" => 0,
            "stdout" => "out",
            "stderr" => "err",
            "cache" => %{"outcome" => "disabled", "reason" => ""}
          }
        ]
      })

    binary =
      fake_engine(root, "mkdir \"$4\"; printf '%s' '" <> report <> "' > \"$4/result.json\"")

    handle =
      start(
        H.plan([H.task("expected", "/usr/bin/true", ".")]),
        Keyword.put(opts, :binary, binary)
      )

    assert {:error, %Local.Error{kind: :decode, reason: :task_ids_mismatch}} = Local.await(handle)
  end

  test "cancellation stops services, active health probes and consumers before releasing leases",
       %{root: root, opts: opts} do
    readiness = H.probe("/bin/sh", ["-c", "test -f service.pid"], interval_ms: 10)

    health =
      H.probe("/bin/sh", ["-c", "echo $$ > probe.pid; /bin/sleep 20"],
        interval_ms: 10,
        timeout_ms: 20_000
      )

    service =
      H.task("service", "/bin/sh", ".")
      |> H.args(["-c", "echo $$ > service.pid; while :; do /bin/sleep 0.02; done"])
      |> H.resources(["database"])
      |> H.timeout_ms(15_000)
      |> H.service(readiness, health: health, shutdown_timeout_ms: 100)

    consumer =
      H.task("consumer", "/bin/sh", ".")
      |> H.needs(["service"])
      |> H.args(["-c", "echo $$ > consumer.pid; while :; do /bin/sleep 0.02; done"])
      |> H.timeout_ms(15_000)

    handle = start(H.plan([service, consumer]) |> H.jobs(1) |> H.resource_dir("leases"), opts)

    for file <- ["service.pid", "probe.pid", "consumer.pid"],
        do: wait_until(fn -> written?(Path.join(root, file)) end)

    assert {:ok, %Local.Report{result: %Result{outcome: :cancelled} = result}} =
             Local.cancel(handle)

    assert %Result.ProbeLogs{} = hd(result.tasks).probe_logs
    events = remaining_events(handle, 0)
    assert Enum.any?(events, &match?({:task_state, %{status: :ready}}, &1.kind))
    assert Enum.any?(events, &match?({:task_state, %{status: :cancelled}}, &1.kind))
    assert Enum.any?(events, &match?({:log, %{stream: :probe_stdout}}, &1.kind))
    assert Enum.any?(events, &match?({:log, %{stream: :probe_stderr}}, &1.kind))
    assert List.last(events).kind == {:run_finished, false}

    for file <- ["service.pid", "probe.pid", "consumer.pid"] do
      pid = File.read!(Path.join(root, file)) |> String.trim()
      wait_until(fn -> stopped?(pid) end)
    end

    next =
      H.task("reuse", "/usr/bin/true", ".")
      |> H.resources(["database"])
      |> H.resource_wait_timeout_ms(500)
      |> H.timeout_ms(2000)

    next_handle = start(H.plan([next]) |> H.resource_dir("leases"), opts)
    assert {:ok, %Local.Report{result: %Result{outcome: :succeeded}}} = Local.await(next_handle)
  end

  test "timed-out polling does not accumulate waiters and dead consumers are removed", %{
    root: root,
    opts: opts
  } do
    handle = start(looping_plan(), opts)
    wait_until(fn -> written?(Path.join(root, "parent.pid")) end)
    for _ <- 1..20, do: assert({:error, %Local.Error{kind: :timeout}} = Local.await(handle, 1))
    assert map_size(:sys.get_state(handle.pid).waiters) == 1
    consumer = spawn(fn -> Local.await(handle) end)
    wait_until(fn -> map_size(:sys.get_state(handle.pid).waiters) == 2 end)
    Process.exit(consumer, :kill)
    wait_until(fn -> map_size(:sys.get_state(handle.pid).waiters) == 1 end)
    waiter_a = Task.async(fn -> Local.await(handle) end)
    waiter_b = Task.async(fn -> Local.await(handle) end)
    assert {:ok, result} = Local.cancel(handle)
    assert {:ok, ^result} = Task.await(waiter_a)
    assert {:ok, ^result} = Task.await(waiter_b)
    assert :sys.get_state(handle.pid).waiters == %{}
  end

  test "a terminated execution handle is not restarted or replayed", %{root: root, opts: opts} do
    task =
      H.task("once", "/usr/bin/true", ".") |> H.env(%{"TEST_SECRET" => "do-not-log-task-value"})

    options = Keyword.update!(opts, :env, &Map.put(&1, "TEST_SECRET", "do-not-log-engine-value"))
    handle = start(H.plan([task]), options)
    assert {:ok, %Local.Report{result: %Result{outcome: :succeeded}}} = Local.await(handle)
    before = File.ls!(Path.join(root, "runs"))
    monitor = Process.monitor(handle.pid)

    logs =
      ExUnit.CaptureLog.capture_log(fn ->
        Process.exit(handle.pid, :kill)
        assert_receive {:DOWN, ^monitor, :process, _, :killed}, 5000

        wait_until(fn ->
          DynamicSupervisor.which_children(FluentCI.Host.Local.Supervisor)
          |> Enum.all?(fn {_, pid, _, _} -> pid != handle.pid end)
        end)
      end)

    assert File.ls!(Path.join(root, "runs")) == before
    refute logs =~ "do-not-log-task-value"
    refute logs =~ "do-not-log-engine-value"
    assert {:error, %Local.Error{kind: :closed}} = Local.await(handle)
  end
end
