defmodule FluentCI.MatrixTest do
  use ExUnit.Case, async: true
  alias FluentCI.Matrix, as: M
  alias FluentCI.Host, as: H

  test "typed products, filters, stable IDs and joins are pure" do
    {:ok, cases} =
      M.product(M.axis("node", ["22", "24"]), M.axis("suite", ["unit", "integration"]))

    cases =
      cases
      |> M.exclude(&(&1 == {"22", "integration"}))
      |> M.include([M.named("experimental", {"26", "unit"})])

    {:ok, tasks} =
      M.matrix("test", cases, fn {node, suite} ->
        H.task("ignored", "/missing", "/missing") |> H.args([node, suite])
      end)

    assert Enum.map(tasks, & &1.data.id) == [
             "test[node=22,suite=unit]",
             "test[node=24,suite=unit]",
             "test[node=24,suite=integration]",
             "test[experimental]"
           ]

    joined = H.task("join", "/missing", "/missing") |> H.needs(["prepare"]) |> M.after_jobs(tasks)
    assert length(joined.data.needs) == 5
    assert M.parallel(tasks, 2).data.jobs == 2
    assert M.named("a,b=世界", nil).key == "a%2Cb%3D%E4%B8%96%E7%95%8C"
  end

  test "duplicate cases reject without invoking the callback" do
    assert {:error, :duplicate_or_empty_case} =
             M.matrix("test", [M.named("same", 1), M.named("same", 2)], fn _ ->
               flunk("callback invoked")
             end)

    assert {:error, :too_many_cases} =
             M.product(M.axis("a", List.duplicate("x", 65)), M.axis("b", List.duplicate("y", 65)))
  end

  test "the Elixir DSL emits the independent plan import" do
    execution =
      FluentCI.Plugin.run_plan(H.plan([H.task("a", "/missing", ".")]), FluentCI.Plugin.input())

    source = FluentCI.Plugin.Codegen.source([{:matrix, execution}])
    assert source =~ "fn execute_plan(input: String)"
    assert source =~ "unsafe { execute_plan(request.to_string()) }"
    refute source =~ "pipeline.with_exec"
  end

  test "workflow functions preserve immutable task values and output dependencies" do
    original = H.task("produce", "/missing", "/missing")

    produced =
      original
      |> H.retry(3, 10)
      |> H.output("version", "version.txt")
      |> H.artifact("report", "report.txt")
      |> H.resource("checkout")

    consumer =
      H.task("consume", "/missing", "/missing") |> H.output_env("VERSION", produced, "version")

    cleanup = consumer |> H.always()
    refute Map.has_key?(original.data, :policy)
    assert consumer.data.needs == ["produce"]
    assert cleanup.data.policy.when == "always"
    assert cleanup.data.timeout_ms == 30_000
    assert {:ok, _} = H.compile(H.plan([produced, consumer]))
    assert produced.data.policy.artifacts["report"] == "report.txt"
  end

  test "native file tasks and env files compile without performing IO" do
    alias FluentCI.Files, as: F
    original = F.write_bytes("write", "path ' 世界", <<0, 255>>, false)
    configured = F.env_file(original, "settings.env")
    refute Map.has_key?(original.data, :env_files)
    assert configured.data.env_files == ["settings.env"]
    assert configured.data.file_operation.bytes == [0, 255]
    assert {:ok, json} = H.compile(H.plan([configured]))
    assert {:ok, decoded} = JSON.decode(json)
    assert hd(decoded["tasks"])["file_operation"]["op"] == "write_bytes"
    assert hd(decoded["tasks"])["program"] == ""
  end
end
