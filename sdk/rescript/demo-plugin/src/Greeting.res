@val @scope("Host") external input: unit => string = "inputString"
@val @scope("Host") external output: string => unit = "outputString"

let greet = () => {
  let result = Guest.pipeline("rescript-greeting")
    ->Guest.exec(["/usr/bin/printf", "%s", input()])
    ->Guest.stdout
  switch result {
  | Ok(value) => output(value)
  | Error(message) => JsError.throwWithMessage(message)
  }
}

let matrix = () => {
  let prepare = Matrix.task("prepare", "/usr/bin/python3", ["-c", "from pathlib import Path; Path('seed.txt').write_text('seed')"], ".")
    ->Matrix.cached(["mode.txt"], ["seed.txt"], "matrix-prepare-v1")
  let cases = switch Matrix.product(Matrix.axis("node", ["22", "24"]), Matrix.axis("suite", ["unit", "integration"])) {
  | Ok(value) => value
  | Error(error) => JsError.throwWithMessage(error)
  }
  let cases = Matrix.exclude(cases, pair => { let (node, suite) = pair; node == "22" && suite == "integration" })
  let cases = Matrix.includeCases(cases, [Matrix.named("experimental", ("26", "unit"))])
  let jobs = switch Matrix.matrix("test", cases, pair => {
    let (node, suite) = pair
    Matrix.task("ignored", "/usr/bin/python3", ["job.py", node, suite], ".")
    ->Matrix.cached(["seed.txt", "mode.txt", "job.py"], ["out-" ++ node ++ "-" ++ suite], "matrix-v1")
    ->Matrix.afterJobs([prepare])
  }) {
  | Ok(value) => value
  | Error(error) => JsError.throwWithMessage(error)
  }
  let join = Matrix.task("join", "/usr/bin/python3", ["-c", "from pathlib import Path; Path('joined').write_text('done')"], ".")
    ->Matrix.afterJobs([prepare])->Matrix.afterJobs(jobs)
  let plan = Matrix.parallel(Array.concat([prepare], Array.concat(jobs, [join])), 2)->Matrix.withFailFast(false)->Matrix.withCacheDirectory("action-cache")
  switch Matrix.run(plan, input()) {
  | Ok(report) => if report.outcome == "succeeded" { output("matrix complete") } else { JsError.throwWithMessage("matrix jobs failed; see the run result") }
  | Error(error) => JsError.throwWithMessage(error)
  }
}
