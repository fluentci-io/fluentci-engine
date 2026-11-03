defmodule DemoPlugin do
  use FluentCI.Plugin

  defplugin greet do
    pipeline("elixir-greeting")
    |> exec(["/usr/bin/printf", "%s", input()])
    |> stdout()
  end

  defplugin matrix do
    alias FluentCI.Host, as: H
    alias FluentCI.Matrix, as: M

    prepare =
      H.task("prepare", "/usr/bin/python3", ".")
      |> H.args(["-c", "from pathlib import Path; Path('seed.txt').write_text('seed')"])

    {:ok, cases} =
      M.product(M.axis("node", ["22", "24"]), M.axis("suite", ["unit", "integration"]))

    cases =
      cases
      |> M.exclude(&(&1 == {"22", "integration"}))
      |> M.include([M.named("experimental", {"26", "unit"})])

    {:ok, jobs} =
      M.matrix("test", cases, fn {node, suite} ->
        H.task("ignored", "/usr/bin/python3", ".")
        |> H.args(["job.py", node, suite])
        |> H.cache(["seed.txt", "mode.txt", "job.py"], ["out-#{node}-#{suite}"], "matrix-v1")
        |> M.after_jobs([prepare])
      end)

    join =
      H.task("join", "/usr/bin/python3", ".")
      |> H.args(["-c", "from pathlib import Path; Path('joined').write_text('done')"])
      |> M.after_jobs([prepare])
      |> M.after_jobs(jobs)

    run_plan(M.parallel([prepare | jobs] ++ [join], 2) |> H.fail_fast(false), input())
  end
end
