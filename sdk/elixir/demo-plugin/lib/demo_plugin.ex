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
      |> H.cache(["mode.txt"], ["seed.txt"], "matrix-prepare-v1")

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

    run_plan(
      M.parallel([prepare | jobs] ++ [join], 2)
      |> H.fail_fast(false)
      |> H.cache_dir("action-cache"),
      input()
    )
  end

  defplugin workflow do
    alias FluentCI.Host, as: H
    alias FluentCI.Matrix, as: M
    task = fn name, mode -> H.task(name, "/usr/bin/python3", ".") |> H.args(["job.py", mode]) end

    producer =
      task.("produce", "produce")
      |> H.retry(3, 10)
      |> H.output("version", "value.txt")
      |> H.artifact("binary", "artifact.bin")
      |> H.resource("checkout")

    consumer = task.("consume", "consume") |> H.output_env("VALUE", producer, "version")
    optional = task.("optional", "optional") |> H.allow_failure()
    never = task.("never", "never") |> H.when(:never)
    failure = task.("failure", "failure") |> M.after_jobs([consumer]) |> H.when(:failure)

    cleanup =
      task.("cleanup", "cleanup")
      |> M.after_jobs([consumer, optional, never, failure])
      |> H.always()

    run_plan(M.parallel([producer, consumer, optional, never, failure, cleanup], 2), input())
  end
end
