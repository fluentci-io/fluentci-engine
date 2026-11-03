defmodule FluentCI.EngineTest do
  use ExUnit.Case, async: true
  alias FluentCI.Host, as: H

  @engine System.get_env(
            "FLUENTCI_ENGINE_BIN",
            Path.expand("../../../target/release/fluentci-engine", __DIR__)
          )
  if not File.regular?(@engine) do
    if System.get_env("FLUENTCI_REQUIRE_ENGINE") == "1",
      do: raise("release engine is required: #{@engine}")

    @moduletag skip: "build the release engine or set FLUENTCI_ENGINE_BIN"
  end

  @moduletag timeout: 30_000
  setup do
    root =
      Path.join(
        System.tmp_dir!(),
        "fluentci-elixir-#{System.unique_integer([:positive, :monotonic])}-#{System.pid()}"
      )

    File.mkdir!(root)
    File.chmod!(root, 0o700)
    on_exit(fn -> File.rm_rf!(root) end)
    {:ok, root: root}
  end

  defp execute(plan, root, name) do
    {:ok, json} = H.compile(plan)
    path = Path.join(root, "#{name}.json")
    File.write!(path, json)
    run = Path.join(root, name)
    # Ambient telemetry must not send integration-test data to external collectors.
    unset = for {key, _} <- System.get_env(), String.starts_with?(key, "OTEL_"), do: {key, nil}

    {output, code} =
      System.cmd(@engine, ["execute", path, "--run-dir", run, "--progress", "plain"],
        env: unset,
        stderr_to_stdout: true
      )

    assert File.regular?(Path.join(run, "result.json")), output
    {JSON.decode!(File.read!(Path.join(run, "result.json"))), run, code}
  end

  test "release engine runs copied workspaces and restores cached outputs and exact logs", %{
    root: root
  } do
    source = Path.join(root, "source")
    File.mkdir!(source)
    File.write!(Path.join(source, "input"), "artifact")
    value = "literal \" 雪\n$(touch unexpected)"

    build =
      H.task("build", "/bin/sh", ".")
      |> H.args(["-c", "/bin/cat input > output; printf '%s' \"$TEXT\""])
      |> H.env(%{"TEXT" => value})
      |> H.timeout_ms(5000)
      |> H.cache(["input"], ["output"], "fixture", relocatable: true)

    consume =
      H.task("consume", "/bin/cat", ".")
      |> H.args(["output"])
      |> H.needs(["build"])
      |> H.timeout_ms(5000)

    plan =
      H.plan([build, consume])
      |> H.workspace(source, ["input"])
      |> H.cache_dir(Path.join(root, "cache"))

    for {name, expected} <- [{"cold", "stored"}, {"warm", "hit"}] do
      {result, run, code} = execute(plan, root, name)
      assert code == 0
      assert result["outcome"] == "succeeded"
      [first, second] = result["tasks"]
      assert first["cache"]["outcome"] == expected
      assert File.read!(first["stdout"]) == value
      assert File.read!(second["stdout"]) == "artifact"
      assert File.read!(Path.join(run, "workspace/output")) == "artifact"
      refute File.exists?(Path.join(run, "workspace/unexpected"))
    end

    assert File.read!(Path.join(source, "input")) == "artifact"
    refute File.exists?(Path.join(source, "output"))
  end

  test "failed dependencies are skipped while independent work completes", %{root: root} do
    fail = H.task("fail", "/bin/sh", root) |> H.args(["-c", "exit 7"]) |> H.timeout_ms(5000)
    child = H.task("child", "/usr/bin/true", root) |> H.needs(["fail"])
    independent = H.task("independent", "/usr/bin/true", root)

    {result, _, code} =
      H.plan([fail, child, independent]) |> H.fail_fast(false) |> execute(root, "failed")

    assert code != 0
    assert result["outcome"] == "failed"
    assert Enum.map(result["tasks"], & &1["status"]) == ["failed", "skipped", "succeeded"]
    assert hd(result["tasks"])["exit_code"] == 7
  end

  test "readiness and health probes gate consumers and service teardown", %{root: root} do
    ready = H.probe("/bin/sh", ["-c", "test -f ready"], interval_ms: 10, timeout_ms: 500)

    db =
      H.task("db", "/bin/sh", root)
      |> H.args(["-c", "printf ready > ready; while :; do /bin/sleep 0.02; done"])
      |> H.timeout_ms(5000)
      |> H.service(ready,
        health: H.probe("/bin/sh", ["-c", "printf healthy > healthy"], interval_ms: 10),
        startup_timeout_ms: 2000,
        shutdown_timeout_ms: 100
      )

    consumer =
      H.task("consumer", "/bin/sh", root)
      |> H.args(["-c", "while [ ! -f healthy ]; do /bin/sleep 0.02; done; /bin/cat ready"])
      |> H.needs(["db"])
      |> H.timeout_ms(2000)

    {result, run, code} = H.plan([db, consumer]) |> H.jobs(1) |> execute(root, "service")
    assert code == 0
    assert result["outcome"] == "succeeded"
    assert File.read!(Enum.at(result["tasks"], 1)["stdout"]) == "ready"
    assert File.read!(Path.join(root, "healthy")) == "healthy"

    events =
      File.read!(Path.join(run, "events.jsonl"))
      |> String.split("\n", trim: true)
      |> Enum.map(&JSON.decode!/1)

    assert Enum.any?(events, &(&1["id"] == "db" and &1["status"] == "ready"))
    assert Enum.any?(events, &(&1["id"] == "db" and &1["status"] == "stopping"))
  end
end
