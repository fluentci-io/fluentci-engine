defmodule FluentCI.ResultTest do
  use ExUnit.Case, async: true
  alias FluentCI.Host.Result
  alias FluentCI.Host.Result.{Cache, Error, ProbeLogs, Task}

  defp task(status \\ "succeeded") do
    %{
      "id" => "build",
      "status" => status,
      "exit_code" => 0,
      "error" => nil,
      "duration_ms" => 42,
      "stdout" => "/run/stdout",
      "stderr" => "/run/stderr",
      "cache" => %{
        "outcome" => "hit",
        "key" => "key",
        "reason" => "verified",
        "components" => %{"inputs" => "digest"}
      },
      "wait_reason" => nil
    }
  end

  defp report(task), do: %{"tasks" => [task]}

  test "terminal variants decode to fixed atoms and preserve diagnostics" do
    for {wire, status} <- [
          {"succeeded", :succeeded},
          {"failed", :failed},
          {"skipped", :skipped},
          {"cancelled", :cancelled},
          {"timed_out", :timed_out}
        ] do
      value =
        task(wire)
        |> Map.merge(%{
          "error" => "diagnostic 雪",
          "exit_code" => nil,
          "probe_logs" => %{"stdout" => "/run/probe-out", "stderr" => "/run/probe-err"},
          "wait_reason" => "resource lease"
        })

      assert {:ok, %Result{outcome: nil, tasks: [decoded]}} =
               Result.decode(JSON.encode!(report(value)))

      assert %Task{status: ^status, duration_ms: 42, exit_code: nil, error: "diagnostic 雪"} =
               decoded

      assert %Cache{outcome: "hit", key: "key", components: %{"inputs" => "digest"}} =
               decoded.cache

      assert %ProbeLogs{stdout: "/run/probe-out", stderr: "/run/probe-err"} = decoded.probe_logs
      assert decoded.wait_reason == "resource lease"
    end
  end

  test "outcome distinguishes fail-fast cancellation from external cancellation" do
    tasks = [task("failed"), task("cancelled") |> Map.put("id", "sibling")]

    for {wire, expected} <- [{"failed", :failed}, {"cancelled", :cancelled}] do
      assert {:ok, %Result{outcome: ^expected}} =
               Result.from_map(%{"tasks" => tasks, "outcome" => wire})
    end

    assert {:ok, legacy} = Result.from_map(%{"tasks" => tasks})
    assert legacy.outcome == nil
    refute Result.succeeded?(legacy)
    assert {:ok, empty} = Result.from_map(%{"tasks" => [], "outcome" => "succeeded"})
    assert Result.succeeded?(empty)
    assert {:ok, success} = Result.from_map(%{"tasks" => [task()], "outcome" => "succeeded"})
    assert Result.succeeded?(success)

    for value <- [
          %{"tasks" => tasks, "outcome" => "succeeded"},
          %{"tasks" => [], "outcome" => "failed"},
          %{"tasks" => [task()], "outcome" => "cancelled"}
        ] do
      assert {:error, %Error{path: ["outcome"], reason: :inconsistent_outcome}} =
               Result.from_map(value)
    end
  end

  test "older optional fields and unknown future fields are supported" do
    old =
      task()
      |> Map.drop(["exit_code", "error", "wait_reason"])
      |> put_in(["cache"], %{"outcome" => "future-cache-mode", "reason" => "reason"})
      |> Map.put("future_field", %{"anything" => true})

    assert {:ok, %Result{outcome: nil, tasks: [decoded]}} = Result.from_map(report(old))
    assert decoded.exit_code == nil
    assert decoded.error == nil
    assert decoded.probe_logs == nil
    assert decoded.wait_reason == nil
    assert decoded.cache.key == nil
    assert decoded.cache.components == %{}
    assert decoded.cache.outcome == "future-cache-mode"
  end

  test "malformed, nonterminal and unknown reports fail with field paths" do
    for status <- ["pending", "running", "ready", "stopping", "future-status", true, nil, %{}] do
      assert {:error, %Error{path: ["tasks", 0, "status"]}} =
               Result.from_map(report(Map.put(task(), "status", status)))
    end

    assert {:error, %Error{path: ["tasks", 1, "id"], reason: :duplicate_id}} =
             Result.from_map(%{"tasks" => [task(), task()]})

    for {field, invalid} <- [
          {"id", ""},
          {"id", <<255>>},
          {"duration_ms", -1},
          {"duration_ms", 1.5},
          {"duration_ms", 340_282_366_920_938_463_463_374_607_431_768_211_456},
          {"exit_code", 2_147_483_648},
          {"exit_code", true},
          {"error", 42},
          {"stdout", nil},
          {"stderr", false},
          {"cache", []},
          {"probe_logs", "path"},
          {"wait_reason", %{}}
        ] do
      assert {:error, %Error{path: ["tasks", 0, ^field]}} =
               Result.from_map(report(Map.put(task(), field, invalid)))
    end

    assert {:error, %Error{path: ["tasks", 0, "probe_logs", "stderr"]}} =
             Result.from_map(report(Map.put(task(), "probe_logs", %{"stdout" => "out"})))

    assert {:error, %Error{path: ["tasks", 0, "cache", "components"]}} =
             Result.from_map(report(put_in(task(), ["cache", "components"], %{"inputs" => 4})))

    for input <- ["{", "null", "[]", "{}", "{\"tasks\":false}", <<255>>, 123] do
      assert {:error, %Error{}} = Result.decode(input)
    end
  end

  test "errors do not echo report values and large durations retain integer precision" do
    value = task() |> Map.put("status", "private-secret-variant")
    assert {:error, error} = Result.from_map(report(value))
    refute inspect(error) =~ "private-secret"

    for large <- [18_446_744_073_709_551_616, 340_282_366_920_938_463_463_374_607_431_768_211_455] do
      assert {:ok, %Result{tasks: [%Task{duration_ms: ^large}]}} =
               task()
               |> Map.put("duration_ms", large)
               |> report()
               |> JSON.encode!()
               |> Result.decode()
    end
  end
end
