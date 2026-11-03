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
end
