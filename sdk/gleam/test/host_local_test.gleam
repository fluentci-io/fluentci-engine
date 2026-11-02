import fluentci/host
import fluentci/host/local
import fluentci/host/result as report
import gleam/javascript/promise.{type Promise}
import gleam/option.{Some}
import gleeunit/should

@external(javascript, "./host_local_test_ffi.mjs", "fixture")
fn fixture(run: fn(String, String) -> Promise(Nil)) -> Promise(Nil)

@external(javascript, "./host_local_test_ffi.mjs", "ready")
fn ready(root: String) -> Promise(Bool)

@external(javascript, "./host_local_test_ffi.mjs", "dead")
fn dead(root: String) -> Bool

pub fn local_success_and_failure_test() {
  fixture(fn(binary, root) {
    let config = local.Config(binary, root, local.Plain)
    let task =
      host.task("ok", "/bin/sh", root)
      |> host.with_args(["-c", "printf hello"])
    let assert Ok(execution) = local.start(host.plan([task]), config)
    use value <- promise.await(local.wait(execution))
    let assert Ok(value) = value
    value.result.outcome |> should.equal(Some(report.Succeeded))
    report.succeeded(value.result) |> should.be_true
    let assert [task_result] = value.result.tasks
    task_result.exit_code |> should.equal(Some(0))
    let task =
      host.task("fail", "/bin/sh", root)
      |> host.with_args(["-c", "exit 17"])
    let assert Ok(execution) = local.start(host.plan([task]), config)
    use value <- promise.map(local.wait(execution))
    let assert Ok(value) = value
    value.result.outcome |> should.equal(Some(report.Failed))
    let assert [task_result] = value.result.tasks
    task_result.exit_code |> should.equal(Some(17))
  })
}

pub fn local_cancellation_waits_for_cleanup_test() {
  fixture(fn(binary, root) {
    let task =
      host.task("long", "/bin/sh", root)
      |> host.with_args(["-c", "echo $$ > pid; while :; do /bin/sleep 1; done"])
      |> host.with_timeout(10_000)
    let assert Ok(execution) =
      local.start(host.plan([task]), local.Config(binary, root, local.Plain))
    use started <- promise.await(ready(root))
    use _ <- promise.await(local.cancel(execution))
    started |> should.be_true
    dead(root) |> should.be_true
    use value <- promise.map(local.wait(execution))
    let assert Ok(value) = value
    value.result.outcome |> should.equal(Some(report.Cancelled))
    let assert [task_result] = value.result.tasks
    task_result.status |> should.equal(report.TaskCancelled)
  })
}

pub fn local_launch_error_test() {
  fixture(fn(_, root) {
    let task = host.task("ok", "/bin/true", root)
    let assert Ok(execution) =
      local.start(
        host.plan([task]),
        local.Config(root <> "/missing", root, local.Plain),
      )
    use value <- promise.map(local.wait(execution))
    let assert Error(local.TransportError(_)) = value
    Nil
  })
}

pub fn terminal_report_validation_test() {
  let task =
    "{\"id\":\"a\",\"status\":\"succeeded\",\"duration_ms\":0,\"stdout\":\"out\",\"stderr\":\"err\",\"cache\":{\"outcome\":\"disabled\",\"reason\":\"none\"}}"
  let assert Ok(_) = report.parse("{\"tasks\":[" <> task <> "]}")
  report.parse("{\"tasks\":[" <> task <> "," <> task <> "]}")
  |> should.equal(Error(Nil))
  report.parse("{\"outcome\":\"failed\",\"tasks\":[" <> task <> "]}")
  |> should.equal(Error(Nil))
  report.parse("{\"outcome\":\"unknown\",\"tasks\":[]}")
  |> should.equal(Error(Nil))
  report.parse("{\"tasks\":[{\"id\":\"a\",\"status\":\"running\"}]}")
  |> should.equal(Error(Nil))
  report.parse("{\"tasks\":42}") |> should.equal(Error(Nil))
  report.parse("invalid") |> should.equal(Error(Nil))
}

pub fn cancellation_before_launch_test() {
  fixture(fn(binary, root) {
    let task = host.task("never", "/bin/true", root)
    let assert Ok(execution) =
      local.start(host.plan([task]), local.Config(binary, root, local.Plain))
    use _ <- promise.await(local.cancel(execution))
    use value <- promise.map(local.wait(execution))
    let assert Error(local.TransportError("execution cancelled before launch")) =
      value
    Nil
  })
}
