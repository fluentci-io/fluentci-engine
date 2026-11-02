/// Authenticated host run API for the JavaScript target (Node/Deno).
import fluentci/host
import fluentci/host/journal
import fluentci/host/result as report
import gleam/dynamic.{type DecodeErrors, type Dynamic}
import gleam/javascript/promise.{type Promise}
import gleam/option.{type Option, None, Some}
import gleam/result

pub type Config {
  Config(base_url: String, token: String, timeout_ms: Int)
}

pub type RunStatus {
  Running
  Succeeded
  Failed
  Cancelled
  Interrupted
}

pub type RunState {
  RunState(
    id: String,
    status: RunStatus,
    error: Option(String),
    created_at_ms: Option(Int),
  )
}

pub type Error {
  InvalidPlan(host.PlanError)
  TransportError(String)
  HttpError(status: Int, body: String)
  DecodeError
  IdempotencyNotAcknowledged
}

@external(javascript, "../../host_client_ffi.mjs", "request")
fn request(
  config: Config,
  method: String,
  id: String,
  suffix: String,
  body: String,
  key: String,
  ok: fn(Int, String, String) -> Result(a, Error),
  error: fn(String) -> Result(a, Error),
) -> Promise(Result(a, Error))

@external(javascript, "../../host_result_ffi.mjs", "parse_json")
fn parse_json(
  text: String,
  ok: fn(Dynamic) -> Result(a, Error),
  error: fn() -> Result(a, Error),
) -> Result(a, Error)

fn invalid() -> Result(a, DecodeErrors) {
  Error([
    dynamic.DecodeError(
      expected: "host run state",
      found: "invalid value",
      path: [],
    ),
  ])
}

fn status(data: Dynamic) -> Result(RunStatus, DecodeErrors) {
  use value <- result.try(dynamic.string(data))
  case value {
    "running" -> Ok(Running)
    "succeeded" -> Ok(Succeeded)
    "failed" -> Ok(Failed)
    "cancelled" -> Ok(Cancelled)
    "interrupted" -> Ok(Interrupted)
    _ -> invalid()
  }
}

fn state(data: Dynamic) -> Result(RunState, DecodeErrors) {
  use id <- result.try(dynamic.field("id", dynamic.string)(data))
  use status <- result.try(dynamic.field("status", status)(data))
  use error <- result.try(dynamic.optional_field("error", dynamic.string)(data))
  use created_at_ms <- result.try(dynamic.optional_field(
    "created_at_ms",
    dynamic.int,
  )(data))
  let valid_time = case created_at_ms {
    None -> True
    Some(value) -> value >= 0 && value <= 9_007_199_254_740_991
  }
  case valid_id(id) && valid_time {
    True -> Ok(RunState(id:, status:, error:, created_at_ms:))
    False -> invalid()
  }
}

@external(javascript, "../../host_client_ffi.mjs", "valid_id")
fn valid_id(id: String) -> Bool

fn decode_state(body: String) -> Result(RunState, Error) {
  parse_json(
    body,
    fn(data) { state(data) |> result.map_error(fn(_) { DecodeError }) },
    fn() { Error(DecodeError) },
  )
}

/// Compile and submit once. Transport failure can mean admission occurred.
pub fn submit(
  plan: host.Plan,
  config: Config,
) -> Promise(Result(RunState, Error)) {
  submit_impl(plan, config, "")
}

/// Reuse the same nonempty key and plan to recover an ambiguous submission.
/// The response must acknowledge the key. No requests are retried automatically.
pub fn submit_with_key(
  plan: host.Plan,
  config: Config,
  key: String,
) -> Promise(Result(RunState, Error)) {
  case key {
    "" ->
      promise.resolve(
        Error(TransportError("idempotency key must not be empty")),
      )
    _ -> submit_impl(plan, config, key)
  }
}

fn submit_impl(
  plan: host.Plan,
  config: Config,
  key: String,
) -> Promise(Result(RunState, Error)) {
  case host.compile(plan) {
    Error(error) -> promise.resolve(Error(InvalidPlan(error)))
    Ok(body) ->
      request(
        config,
        "POST",
        "",
        "",
        body,
        key,
        fn(code, body, acknowledged) {
          case code {
            202 ->
              case acknowledged == key {
                True -> decode_state(body)
                False -> Error(IdempotencyNotAcknowledged)
              }
            _ -> Error(HttpError(code, body))
          }
        },
        fn(message) { Error(TransportError(message)) },
      )
  }
}

pub fn get_state(
  id: String,
  config: Config,
) -> Promise(Result(RunState, Error)) {
  request(
    config,
    "GET",
    id,
    "",
    "",
    "",
    fn(code, body, _) {
      case code {
        200 -> {
          use state <- result.try(decode_state(body))
          case state.id == id {
            True -> Ok(state)
            False -> Error(DecodeError)
          }
        }
        _ -> Error(HttpError(code, body))
      }
    },
    fn(message) { Error(TransportError(message)) },
  )
}

/// A running run can return HTTP 404 until its terminal report is published.
pub fn get_result(
  id: String,
  config: Config,
) -> Promise(Result(report.RunResult, Error)) {
  request(
    config,
    "GET",
    id,
    "/result",
    "",
    "",
    fn(code, body, _) {
      case code {
        200 -> report.parse(body) |> result.map_error(fn(_) { DecodeError })
        _ -> Error(HttpError(code, body))
      }
    },
    fn(message) { Error(TransportError(message)) },
  )
}

/// Request cancellation. Acceptance does not mean process cleanup has finished;
/// poll get_state until terminal before consuming get_result.
pub fn cancel(id: String, config: Config) -> Promise(Result(Nil, Error)) {
  request(
    config,
    "POST",
    id,
    "/cancel",
    "",
    "",
    fn(code, body, _) {
      case code {
        202 -> Ok(Nil)
        _ -> Error(HttpError(code, body))
      }
    },
    fn(message) { Error(TransportError(message)) },
  )
}

pub type LogStream {
  Stdout
  Stderr
  ProbeStdout
  ProbeStderr
}

pub type LogPage {
  LogPage(bytes: BitArray, next_cursor: Int)
}

@external(javascript, "../../host_client_ffi.mjs", "logs")
fn logs_impl(
  config: Config,
  id: String,
  index: Int,
  stream: String,
  cursor: Int,
  ok: fn(Dynamic, Int) -> Result(LogPage, Error),
  invalid: fn() -> Result(LogPage, Error),
  http_error: fn(Int, String) -> Result(LogPage, Error),
  transport_error: fn(String) -> Result(LogPage, Error),
) -> Promise(Result(LogPage, Error))

/// Read up to 1 MiB of raw output from a zero-based task index and byte cursor.
/// Preserve next_cursor for reconnection. An empty page is not proof of run completion.
/// Probe streams can return HTTP 404 until the first probe starts.
pub fn get_logs(
  id: String,
  index: Int,
  stream: LogStream,
  cursor: Int,
  config: Config,
) -> Promise(Result(LogPage, Error)) {
  let stream = case stream {
    Stdout -> "stdout"
    Stderr -> "stderr"
    ProbeStdout -> "probe_stdout"
    ProbeStderr -> "probe_stderr"
  }
  logs_impl(
    config,
    id,
    index,
    stream,
    cursor,
    fn(data, next_cursor) {
      dynamic.bit_array(data)
      |> result.map_error(fn(_) { DecodeError })
      |> result.map(fn(bytes) { LogPage(bytes:, next_cursor:) })
    },
    fn() { Error(DecodeError) },
    fn(code, body) { Error(HttpError(code, body)) },
    fn(message) { Error(TransportError(message)) },
  )
}

pub type EventPage {
  EventPage(events: List(journal.Event), next_cursor: Int)
}

@external(javascript, "../../host_client_ffi.mjs", "events")
fn events_impl(
  config: Config,
  id: String,
  cursor: Int,
  ok: fn(String, Int) -> Result(EventPage, Error),
  invalid: fn() -> Result(EventPage, Error),
  http_error: fn(Int, String) -> Result(EventPage, Error),
  transport_error: fn(String) -> Result(EventPage, Error),
) -> Promise(Result(EventPage, Error))

/// Read up to 4 MiB of complete journal records. Start at zero and resume with
/// the returned byte cursor, not a sequence number. Empty pages may precede completion.
pub fn get_events(
  id: String,
  cursor: Int,
  config: Config,
) -> Promise(Result(EventPage, Error)) {
  events_impl(
    config,
    id,
    cursor,
    fn(text, next_cursor) {
      journal.parse(text)
      |> result.map_error(fn(_) { DecodeError })
      |> result.map(fn(events) { EventPage(events:, next_cursor:) })
    },
    fn() { Error(DecodeError) },
    fn(code, body) { Error(HttpError(code, body)) },
    fn(message) { Error(TransportError(message)) },
  )
}
