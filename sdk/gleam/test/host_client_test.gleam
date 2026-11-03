import fluentci/host
import fluentci/host/client
import fluentci/host/journal
import fluentci/host/result as report
import gleam/bit_array
import gleam/javascript/promise.{type Promise}
import gleam/list
import gleam/option.{None, Some}
import gleam/string
import gleeunit/should

@external(javascript, "./host_client_test_ffi.mjs", "fixture")
fn fixture(run: fn(String, String, String) -> Promise(Nil)) -> Promise(Nil)

@external(javascript, "./host_client_test_ffi.mjs", "pause")
fn pause() -> Promise(Nil)

@external(javascript, "./host_client_test_ffi.mjs", "transport_contract")
fn transport_contract() -> Promise(Nil)

fn terminal(
  id: String,
  config: client.Config,
  attempts: Int,
) -> Promise(client.RunState) {
  use value <- promise.await(client.get_state(id, config))
  let assert Ok(state) = value
  case state.status {
    client.Queued | client.Running -> {
      let assert True = attempts > 0
      use _ <- promise.await(pause())
      terminal(id, config, attempts - 1)
    }
    _ -> promise.resolve(state)
  }
}

pub fn remote_execution_replay_and_cancel_test() {
  fixture(fn(base, token, root) {
    let config = client.Config(base, token, 5000)
    use value <- promise.await(client.history(10, None, config))
    value |> should.equal(Ok(client.RunPage([], None)))
    let probe =
      host.Probe(
        "/bin/sh",
        ["-c", "printf probe; printf diagnostic >&2"],
        10,
        1000,
      )
    let service =
      host.task("service", "/bin/sh", root)
      |> host.with_args(["-c", "while :; do /bin/sleep 1; done"])
      |> host.as_service(host.service(probe))
    let task =
      host.task("build", "/bin/sh", root)
      |> host.with_args([
        "-c",
        "printf executed >> output; printf '\\000\\377\\303\\251'",
      ])
      |> host.needs([service])
    let plan = host.plan([service, task])
    use value <- promise.await(client.submit_with_key(
      plan,
      config,
      "gleam-replay",
    ))
    let assert Ok(admitted) = value
    use value <- promise.await(client.submit_with_key(
      plan,
      config,
      "gleam-replay",
    ))
    let assert Ok(replayed) = value
    replayed.id |> should.equal(admitted.id)
    use state <- promise.await(terminal(admitted.id, config, 200))
    state.status |> should.equal(client.Succeeded)
    let succeeded_id = admitted.id
    use value <- promise.await(client.get_result(admitted.id, config))
    let assert Ok(value) = value
    value.outcome |> should.equal(Some(report.Succeeded))
    use value <- promise.await(client.get_logs(
      admitted.id,
      1,
      client.Stdout,
      0,
      config,
    ))
    let assert Ok(page) = value
    page.bytes |> should.equal(<<0, 255, 195, 169>>)
    page.next_cursor |> should.equal(4)
    use value <- promise.await(client.get_logs(
      admitted.id,
      1,
      client.Stdout,
      3,
      config,
    ))
    let assert Ok(page) = value
    page.bytes |> should.equal(<<169>>)
    page.next_cursor |> should.equal(4)
    use value <- promise.await(client.get_logs(
      admitted.id,
      1,
      client.Stdout,
      page.next_cursor,
      config,
    ))
    let assert Ok(page) = value
    page.bytes |> should.equal(<<>>)
    page.next_cursor |> should.equal(4)
    use value <- promise.await(client.get_logs(
      admitted.id,
      0,
      client.ProbeStdout,
      0,
      config,
    ))
    let assert Ok(page) = value
    let assert Ok(probe_text) = bit_array.to_string(page.bytes)
    string.contains(probe_text, "probe") |> should.be_true
    use value <- promise.await(client.get_logs(
      admitted.id,
      0,
      client.ProbeStderr,
      0,
      config,
    ))
    let assert Ok(page) = value
    let assert Ok(probe_text) = bit_array.to_string(page.bytes)
    string.contains(probe_text, "diagnostic") |> should.be_true
    use value <- promise.await(client.get_logs(
      admitted.id,
      1,
      client.Stdout,
      5,
      config,
    ))
    let assert Error(client.HttpError(400, _)) = value
    use value <- promise.await(client.get_events(admitted.id, 0, config))
    let assert Ok(page) = value
    let assert Ok(first) = list.first(page.events)
    first.kind |> should.equal(journal.RunStarted(2))
    let assert Ok(last) = list.last(page.events)
    last.kind |> should.equal(journal.RunFinished(True))
    list.any(page.events, fn(event) {
      case event.kind {
        journal.Log("service", journal.ProbeStdout, _) -> True
        _ -> False
      }
    })
    |> should.be_true
    use value <- promise.await(client.get_events(
      admitted.id,
      page.next_cursor,
      config,
    ))
    value |> should.equal(Ok(client.EventPage([], page.next_cursor)))
    let changed = host.plan([host.task("other", "/bin/true", root)])
    use value <- promise.await(client.submit_with_key(
      changed,
      config,
      "gleam-replay",
    ))
    let assert Error(client.HttpError(409, _)) = value
    let long =
      host.task("long", "/bin/sh", root)
      |> host.with_args(["-c", "printf live; while :; do /bin/sleep 1; done"])
      |> host.with_timeout(10_000)
    use value <- promise.await(client.submit(host.plan([long]), config))
    let assert Ok(admitted) = value
    use _ <- promise.await(live_output(admitted.id, config, 200))
    use value <- promise.await(client.get_state(admitted.id, config))
    let assert Ok(running) = value
    running.status |> should.equal(client.Running)
    use value <- promise.await(client.get_events(admitted.id, 0, config))
    let assert Ok(live_page) = value
    let assert Ok(last_live) = list.last(live_page.events)
    use value <- promise.await(client.cancel(admitted.id, config))
    value |> should.equal(Ok(Nil))
    use state <- promise.await(terminal(admitted.id, config, 200))
    state.status |> should.equal(client.Cancelled)
    use value <- promise.await(client.get_result(admitted.id, config))
    let assert Ok(value) = value
    value.outcome |> should.equal(Some(report.Cancelled))
    use value <- promise.await(client.get_events(
      admitted.id,
      live_page.next_cursor,
      config,
    ))
    let assert Ok(tail) = value
    let assert Ok(first_tail) = list.first(tail.events)
    { first_tail.sequence > last_live.sequence } |> should.be_true
    let assert Ok(last_tail) = list.last(tail.events)
    last_tail.kind |> should.equal(journal.RunFinished(False))

    use value <- promise.await(client.history(1, None, config))
    let assert Ok(client.RunPage([newest], Some(cursor))) = value
    newest.id |> should.equal(admitted.id)
    newest.status |> should.equal(client.Cancelled)
    let another = host.plan([host.task("newest", "/bin/true", root)])
    use value <- promise.await(client.submit(another, config))
    let assert Ok(added) = value
    use _ <- promise.await(terminal(added.id, config, 200))
    use value <- promise.await(client.history(1, Some(cursor), config))
    let assert Ok(client.RunPage([older], None)) = value
    older.id |> should.equal(succeeded_id)
    older.status |> should.equal(client.Succeeded)
    use value <- promise.await(client.history(1, None, config))
    let assert Ok(client.RunPage([latest], _)) = value
    latest.id |> should.equal(added.id)
    use value <- promise.await(client.history(
      10,
      None,
      client.Config(base, "wrong", 5000),
    ))
    let assert Error(client.HttpError(401, _)) = value
    use value <- promise.map(client.get_state(
      admitted.id,
      client.Config(base, "wrong", 5000),
    ))
    let assert Error(client.HttpError(401, _)) = value
    Nil
  })
}

pub fn http_transport_contract_test() {
  transport_contract()
}

@external(javascript, "./host_client_test_ffi.mjs", "mock_fixture")
fn mock_fixture(run: fn(String) -> Promise(Nil)) -> Promise(Nil)

pub fn remote_rejects_invalid_responses_test() {
  mock_fixture(fn(base) {
    let id = "00000000-0000-0000-0000-000000000001"
    use value <- promise.await(client.get_state(
      id,
      client.Config(base, "queued-state", 1000),
    ))
    let assert Ok(queued) = value
    queued.status |> should.equal(client.Queued)
    use value <- promise.await(client.get_state(
      id,
      client.Config(base, "unknown-status", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    use value <- promise.await(client.get_state(
      id,
      client.Config(base, "wrong-id", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    use value <- promise.await(client.get_state(
      id,
      client.Config(base, "negative-time", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    use value <- promise.await(client.get_state(
      id,
      client.Config(base, "malformed", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    let plan = host.plan([host.task("task", "/bin/true", ".")])
    use value <- promise.map(client.submit_with_key(
      plan,
      client.Config(base, "missing-ack", 1000),
      "key",
    ))
    value |> should.equal(Error(client.IdempotencyNotAcknowledged))
  })
}

fn live_output(
  id: String,
  config: client.Config,
  attempts: Int,
) -> Promise(Nil) {
  use value <- promise.await(client.get_logs(id, 0, client.Stdout, 0, config))
  case value {
    Ok(client.LogPage(bytes, _)) if bytes == <<108, 105, 118, 101>> ->
      promise.resolve(Nil)
    _ -> {
      let assert True = attempts > 0
      use _ <- promise.await(pause())
      live_output(id, config, attempts - 1)
    }
  }
}

@external(javascript, "./host_client_test_ffi.mjs", "log_transport_contract")
fn log_transport_contract() -> Promise(Nil)

pub fn log_cursor_validation_test() {
  log_transport_contract()
}

@external(javascript, "./host_client_test_ffi.mjs", "event_transport_contract")
fn event_transport_contract() -> Promise(Nil)

pub fn event_cursor_validation_test() {
  event_transport_contract()
}

@external(javascript, "./host_client_test_ffi.mjs", "traced_fixture")
fn traced_fixture(
  run: fn(String, String, String) -> Promise(Nil),
) -> Promise(Nil)

@external(javascript, "./host_client_test_ffi.mjs", "trace_transport_contract")
fn trace_transport_contract() -> Promise(Nil)

pub fn trace_header_validation_test() {
  trace_transport_contract()
}

pub fn traced_submission_reaches_events_and_exporter_test() {
  traced_fixture(fn(base, token, root) {
    let config = client.Config(base, token, 5000)
    let trace = "1234567890abcdef1234567890abcdef"
    let context =
      client.TraceContext(
        "00-" <> trace <> "-fedcba9876543210-01",
        "vendor=gleam",
      )
    let task =
      host.task("traced", "/bin/sh", root)
      |> host.with_args(["-c", "printf executed >> output"])
    let plan = host.plan([task])
    use value <- promise.await(client.submit_with_context(
      plan,
      config,
      Some("traced-key"),
      context,
    ))
    let assert Ok(admitted) = value
    use state <- promise.await(terminal(admitted.id, config, 200))
    state.status |> should.equal(client.Succeeded)
    let other =
      client.TraceContext(
        "00-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bbbbbbbbbbbbbbbb-01",
        "vendor=other",
      )
    use value <- promise.await(client.submit_with_context(
      plan,
      config,
      Some("traced-key"),
      other,
    ))
    let assert Ok(replayed) = value
    replayed.id |> should.equal(admitted.id)
    use value <- promise.map(client.get_events(admitted.id, 0, config))
    let assert Ok(page) = value
    { page.events != [] } |> should.be_true
    list.each(page.events, fn(event) {
      event.trace_id |> should.equal(Some(trace))
    })
    let assert Ok(first) = list.first(page.events)
    list.any(page.events, fn(event) { event.span_id != first.span_id })
    |> should.be_true
  })
}

@external(javascript, "./host_client_test_ffi.mjs", "history_transport_contract")
fn history_transport_contract() -> Promise(Nil)

pub fn history_transport_test() {
  history_transport_contract()
}

pub fn history_response_validation_test() {
  mock_fixture(fn(base) {
    use value <- promise.await(client.history(
      10,
      None,
      client.Config(base, "duplicate-history", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    use value <- promise.await(client.history(
      10,
      None,
      client.Config(base, "empty-history-cursor", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    use value <- promise.await(client.history(
      10,
      Some("same"),
      client.Config(base, "repeated-history-cursor", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
    use value <- promise.map(client.history(
      10,
      None,
      client.Config(base, "unknown-history-status", 1000),
    ))
    value |> should.equal(Error(client.DecodeError))
  })
}

@external(javascript, "./host_client_test_ffi.mjs", "fixture")
fn restart_fixture(
  run: fn(String, String, String, fn() -> Promise(Nil)) -> Promise(Nil),
) -> Promise(Nil)

pub fn history_rediscovers_runs_after_restart_test() {
  restart_fixture(fn(base, token, root, restart) {
    let config = client.Config(base, token, 5000)
    let task =
      host.task("persisted", "/bin/sh", root)
      |> host.with_args(["-c", "printf executed >> output"])
    use value <- promise.await(client.submit(host.plan([task]), config))
    let assert Ok(admitted) = value
    use _ <- promise.await(terminal(admitted.id, config, 200))
    use value <- promise.await(client.history(10, None, config))
    let assert Ok(before) = value
    use _ <- promise.await(restart())
    use value <- promise.await(client.history(10, None, config))
    value |> should.equal(Ok(before))
    let assert [saved] = before.runs
    saved.id |> should.equal(admitted.id)
    saved.status |> should.equal(client.Succeeded)
    use value <- promise.map(client.get_result(saved.id, config))
    let assert Ok(value) = value
    value.outcome |> should.equal(Some(report.Succeeded))
  })
}

pub fn queued_runs_can_be_cancelled_before_execution_test() {
  fixture(fn(base, token, root) {
    let config = client.Config(base, token, 5000)
    let blocker =
      host.task("blocker", "/bin/sh", root)
      |> host.with_args([
        "-c",
        "printf executed > output; printf live; while :; do /bin/sleep 0.02; done",
      ])
      |> host.with_timeout(10_000)
    use value <- promise.await(client.submit(host.plan([blocker]), config))
    let assert Ok(running) = value
    use _ <- promise.await(live_output(running.id, config, 200))
    let queued_plan = host.plan([host.task("queued", "/bin/true", root)])
    use value <- promise.await(client.submit(queued_plan, config))
    let assert Ok(queued) = value
    queued.status |> should.equal(client.Queued)
    use value <- promise.await(client.cancel(queued.id, config))
    value |> should.equal(Ok(Nil))
    use state <- promise.await(terminal(queued.id, config, 200))
    state.status |> should.equal(client.Cancelled)
    use value <- promise.await(client.get_result(queued.id, config))
    let assert Error(client.HttpError(404, _)) = value
    use value <- promise.await(client.cancel(running.id, config))
    value |> should.equal(Ok(Nil))
    use state <- promise.map(terminal(running.id, config, 200))
    state.status |> should.equal(client.Cancelled)
  })
}
