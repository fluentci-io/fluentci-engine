defmodule FluentCI.JournalTest do
  use ExUnit.Case, async: true
  alias FluentCI.Host.Journal, as: J

  setup do
    root =
      Path.join(
        System.tmp_dir!(),
        "fluentci-journal-#{System.unique_integer([:positive, :monotonic])}"
      )

    File.mkdir!(root)
    on_exit(fn -> File.rm_rf!(root) end)
    {:ok, root: root, path: Path.join(root, "events.jsonl")}
  end

  defp wire(type, values, sequence \\ 0) do
    Map.merge(%{"type" => type, "sequence" => sequence, "elapsed_ms" => 0}, values)
  end

  defp line(event), do: JSON.encode!(event) <> "\n"

  test "every event variant, stream, status, correlation and full u64 precision" do
    max = 18_446_744_073_709_551_615

    assert {:ok, %J.Event{sequence: ^max, elapsed_ms: ^max, kind: {:run_started, ^max}}} =
             J.decode(
               JSON.encode!(%{
                 "type" => "run_started",
                 "sequence" => max,
                 "elapsed_ms" => max,
                 "tasks" => max
               })
             )

    for status <- ~w(pending running ready stopping succeeded failed skipped cancelled timed_out) do
      event =
        wire("task_state", %{
          "id" => "step",
          "status" => status,
          "cache" => "future-cache-kind",
          "reason" => "",
          "duration_ms" => max,
          "trace_id" => "trace",
          "span_id" => "span",
          "future" => true
        })

      assert {:ok, %J.Event{trace_id: "trace", span_id: "span", kind: {:task_state, task}}} =
               J.from_map(event)

      assert Atom.to_string(task.status) == status
      assert task.duration_ms == max and task.cache == "future-cache-kind"
      assert task.wait_reason == nil and task.error == nil
    end

    for stream <- ~w(stdout stderr probe_stdout probe_stderr) do
      assert {:ok, %J.Event{kind: {:log, log}}} =
               J.from_map(wire("log", %{"id" => "s", "stream" => stream, "text" => "你好\n\u0000"}))

      assert Atom.to_string(log.stream) == stream
      assert log.text == "你好\n\u0000"
    end

    for success <- [true, false] do
      assert {:ok, %J.Event{kind: {:run_finished, ^success}}} =
               J.from_map(wire("run_finished", %{"success" => success}))
    end
  end

  test "malformed data returns fixed errors without retaining input" do
    valid = wire("log", %{"id" => "s", "stream" => "stdout", "text" => "secret"})

    for {key, bad} <- [
          {"sequence", -1},
          {"elapsed_ms", 1.5},
          {"sequence", 18_446_744_073_709_551_616},
          {"id", ""},
          {"stream", "secret-new-atom"},
          {"type", "secret-new-atom"},
          {"text", 1},
          {"trace_id", %{"secret" => true}}
        ] do
      assert {:error, %J.Error{kind: :decode} = error} = J.from_map(Map.put(valid, key, bad))
      refute inspect(error) =~ "secret"
    end

    for bad <- [nil, [], %J.Event{}], do: assert({:error, _} = J.from_map(bad))
    assert {:error, %J.Error{reason: :invalid_json}} = J.decode("secret malformed JSON")
  end

  test "live partial records resume at byte boundaries without duplicates", %{
    root: root,
    path: path
  } do
    assert {:ok, %J.Batch{events: [], next_cursor: 0}} = J.read(root)
    first = line(wire("run_started", %{"tasks" => 1}))
    second = line(wire("log", %{"id" => "s", "stream" => "stdout", "text" => "你好"}, 1))
    File.write!(path, first <> binary_part(second, 0, byte_size(second) - 1))
    assert {:ok, %J.Batch{events: [%J.Event{sequence: 0}], next_cursor: cursor}} = J.read(root)
    assert cursor == byte_size(first)
    assert {:ok, %J.Batch{events: [], next_cursor: ^cursor}} = J.read(root, cursor)
    File.write!(path, "\n" <> line(wire("run_finished", %{"success" => true}, 2)), [:append])

    assert {:ok, %J.Batch{events: [%J.Event{sequence: 1}], next_cursor: next}} =
             J.read(root, cursor, 1)

    assert next == byte_size(first) + byte_size(second)

    assert {:ok, %J.Batch{events: [%J.Event{kind: {:run_finished, true}}], next_cursor: last}} =
             J.read(root, next)

    assert {:ok, %J.Batch{events: [], next_cursor: ^last}} = J.read(root, last)
    assert last == File.stat!(path).size
  end

  test "invalid cursors, truncated files and nonregular journals fail", %{root: root, path: path} do
    File.write!(path, line(wire("run_started", %{"tasks" => 0})))
    assert {:error, %J.Error{kind: :cursor, reason: :not_record_boundary}} = J.read(root, 1)
    assert {:ok, batch} = J.read(root)
    File.write!(path, "")
    assert {:error, %J.Error{reason: :beyond_end}} = J.read(root, batch.next_cursor)
    File.rm!(path)
    File.mkdir!(path)
    assert {:error, %J.Error{reason: :nonregular_journal}} = J.read(root)
    File.rmdir!(path)
    File.ln_s!(Path.join(root, "missing"), path)
    assert {:error, %J.Error{reason: :nonregular_journal}} = J.read(root)
    File.rm!(path)
    {_, 0} = System.cmd("mkfifo", [path])
    assert {:error, %J.Error{reason: :nonregular_journal}} = J.read(root)

    for {cursor, limit} <- [{-1, 1}, {0, 0}, {0, 1001}, {1.0, 2}] do
      assert {:error, %J.Error{kind: :options}} = J.read(root, cursor, limit)
    end

    assert {:error, %J.Error{kind: :options}} = J.read("")
  end

  test "page byte limit is resumable and oversized records fail boundedly", %{
    root: root,
    path: path
  } do
    text = String.duplicate("x", 1_500_000)

    records =
      for sequence <- 0..2,
          do: line(wire("log", %{"id" => "s", "stream" => "stdout", "text" => text}, sequence))

    File.write!(path, records)
    assert {:ok, %J.Batch{events: [_, _], next_cursor: cursor}} = J.read(root)
    assert {:ok, %J.Batch{events: [%J.Event{sequence: 2}]}} = J.read(root, cursor)
    File.write!(path, String.duplicate("x", 4 * 1024 * 1024 + 1))
    assert {:error, %J.Error{reason: :record_too_large}} = J.read(root)
  end

  test "a record exactly at the byte cap is returned before the next page", %{
    root: root,
    path: path
  } do
    max = 4 * 1024 * 1024
    event = wire("log", %{"id" => "s", "stream" => "stdout", "text" => ""})
    text = String.duplicate("x", max - byte_size(line(event)))
    record = line(Map.put(event, "text", text))
    assert byte_size(record) == max
    File.write!(path, record <> line(wire("run_finished", %{"success" => true}, 1)))
    assert {:ok, %J.Batch{events: [%J.Event{sequence: 0}], next_cursor: ^max}} = J.read(root)
    assert {:ok, %J.Batch{events: [%J.Event{sequence: 1}]}} = J.read(root, max)
  end

  test "invalid JSON and noncontiguous sequences do not return partial pages", %{
    root: root,
    path: path
  } do
    first = line(wire("run_started", %{"tasks" => 0}))
    File.write!(path, first <> "secret-malformed\n")
    assert {:error, %J.Error{reason: :invalid_json, cursor: cursor} = error} = J.read(root)
    assert cursor == byte_size(first)
    refute inspect(error) =~ "secret"
    File.write!(path, first <> line(wire("run_finished", %{"success" => true}, 3)))
    assert {:error, %J.Error{reason: :sequence_gap}} = J.read(root)
  end
end
