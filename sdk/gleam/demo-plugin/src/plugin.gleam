import fluentci/host
import fluentci/host/files as fs
import fluentci/host/result as reports
import fluentci/matrix as parallel
import fluentci/plugin
import gleam/list
import gleam/result

pub fn greet(input: String) -> Result(String, String) {
  plugin.pipeline("gleam-greeting")
  |> plugin.exec("/usr/bin/printf", ["%s", input])
  |> plugin.stdout
}

pub fn matrix(directory: String) -> Result(reports.RunResult, String) {
  let prepare =
    host.task("prepare", "/usr/bin/python3", ".")
    |> host.with_args([
      "-c",
      "from pathlib import Path; Path('seed.txt').write_text('seed')",
    ])
    |> host.with_cache(host.cache(
      ["mode.txt"],
      ["seed.txt"],
      "matrix-prepare-v1",
    ))
  use cases <- result.try(
    parallel.product(
      parallel.axis("node", ["22", "24"]),
      parallel.axis("suite", ["unit", "integration"]),
    )
    |> result.map_error(fn(_) { "invalid matrix product" }),
  )
  let cases =
    parallel.exclude(cases, fn(pair) { pair == #("22", "integration") })
    |> parallel.include([parallel.named("experimental", #("26", "unit"))])
  use jobs <- result.try(
    parallel.matrix("test", cases, fn(pair) {
      let #(node, suite) = pair
      host.task("ignored", "/usr/bin/python3", ".")
      |> host.with_args(["job.py", node, suite])
      |> host.with_cache(host.cache(
        ["seed.txt", "mode.txt", "job.py"],
        ["out-" <> node <> "-" <> suite],
        "matrix-v1",
      ))
      |> parallel.after_jobs([prepare])
    })
    |> result.map_error(fn(_) { "invalid matrix" }),
  )
  let join =
    host.task("join", "/usr/bin/python3", ".")
    |> host.with_args([
      "-c",
      "from pathlib import Path; Path('joined').write_text('done')",
    ])
    |> parallel.after_jobs([prepare])
    |> parallel.after_jobs(jobs)
  let tasks = list.append([prepare], list.append(jobs, [join]))
  use report <- result.try(
    parallel.parallel(tasks, 2)
    |> host.with_cache_directory("action-cache")
    |> host.with_fail_fast(False)
    |> parallel.run(directory),
  )
  case reports.succeeded(report) {
    True -> Ok(report)
    False -> Error("matrix jobs failed; see the run result")
  }
}

pub fn workflow(directory: String) -> Result(reports.RunResult, String) {
  let task = fn(name, mode) {
    host.task(name, "/usr/bin/python3", ".") |> host.with_args(["job.py", mode])
  }
  let producer =
    task("produce", "produce")
    |> host.retry(3, 10)
    |> host.output("version", "value.txt")
    |> host.artifact("binary", "artifact.bin")
    |> host.resource("checkout")
  let consumer =
    task("consume", "consume") |> host.output_env("VALUE", producer, "version")
  let optional = task("optional", "optional") |> host.allow_failure(True)
  let never = task("never", "never") |> host.when_task(host.Never)
  let failure =
    task("failure", "failure")
    |> parallel.after_jobs([consumer])
    |> host.when_task(host.OnFailure)
  let cleanup =
    task("cleanup", "cleanup")
    |> parallel.after_jobs([consumer, optional, never, failure])
    |> host.always
  use report <- result.try(
    parallel.parallel(
      [producer, consumer, optional, never, failure, cleanup],
      2,
    )
    |> parallel.run(directory),
  )
  case reports.succeeded(report) {
    True -> Ok(report)
    False -> Error("workflow failed; see result.json")
  }
}

pub fn files(directory: String) -> Result(reports.RunResult, String) {
  let dir = fs.mkdir("directory", "data", True, ".")
  let write =
    fs.write_text(
      "write",
      "data/version.env",
      "VERSION='héllo 世界; $(touch injected)'\n",
      False,
      ".",
    )
    |> parallel.after_jobs([dir])
  let copy =
    fs.copy_file("copy", "data/version.env", "copied.env", False, ".")
    |> parallel.after_jobs([write])
  let read =
    fs.read_text("read", "copied.env", 1_048_576, ".")
    |> parallel.after_jobs([copy])
  let hash = fs.hash("hash", "copied.env", ".") |> parallel.after_jobs([copy])
  let env =
    host.task("env", "/usr/bin/printenv", ".")
    |> host.with_args(["VERSION"])
    |> fs.env_file("copied.env")
    |> parallel.after_jobs([copy])
  parallel.parallel([dir, write, copy, read, hash, env], 2)
  |> parallel.run(directory)
}
