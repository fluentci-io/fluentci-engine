import fluentci/host
import fluentci/host/client
import fluentci/host/result as report
import gleam/javascript/promise.{type Promise}
import gleam/option.{Some}
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
    client.Running -> {
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
    let task =
      host.task("build", "/bin/sh", root)
      |> host.with_args(["-c", "printf executed >> output"])
    let plan = host.plan([task])
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
    use value <- promise.await(client.get_result(admitted.id, config))
    let assert Ok(value) = value
    value.outcome |> should.equal(Some(report.Succeeded))
    let changed = host.plan([host.task("other", "/bin/true", root)])
    use value <- promise.await(client.submit_with_key(
      changed,
      config,
      "gleam-replay",
    ))
    let assert Error(client.HttpError(409, _)) = value
    let long =
      host.task("long", "/bin/sh", root)
      |> host.with_args(["-c", "while :; do /bin/sleep 1; done"])
      |> host.with_timeout(10_000)
    use value <- promise.await(client.submit(host.plan([long]), config))
    let assert Ok(admitted) = value
    use value <- promise.await(client.cancel(admitted.id, config))
    value |> should.equal(Ok(Nil))
    use state <- promise.await(terminal(admitted.id, config, 200))
    state.status |> should.equal(client.Cancelled)
    use value <- promise.await(client.get_result(admitted.id, config))
    let assert Ok(value) = value
    value.outcome |> should.equal(Some(report.Cancelled))
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
