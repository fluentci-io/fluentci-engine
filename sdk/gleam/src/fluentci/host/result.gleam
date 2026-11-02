/// Typed terminal reports shared by host execution adapters.
import gleam/dict
import gleam/dynamic.{type DecodeErrors, type Dynamic, DecodeError}
import gleam/list
import gleam/option.{type Option, Some}
import gleam/result

pub type Outcome {
  Succeeded
  Failed
  Cancelled
}

pub type TaskStatus {
  TaskSucceeded
  TaskFailed
  TaskSkipped
  TaskCancelled
  TaskTimedOut
}

pub type Cache {
  Cache(outcome: String, key: Option(String), reason: String)
}

pub type TaskResult {
  TaskResult(
    id: String,
    status: TaskStatus,
    exit_code: Option(Int),
    error: Option(String),
    duration_ms: Int,
    stdout: String,
    stderr: String,
    cache: Cache,
  )
}

pub type RunResult {
  RunResult(outcome: Option(Outcome), tasks: List(TaskResult))
}

fn invalid() -> Result(a, DecodeErrors) {
  Error([
    DecodeError(
      expected: "terminal host result",
      found: "invalid value",
      path: [],
    ),
  ])
}

fn outcome(data: Dynamic) -> Result(Outcome, DecodeErrors) {
  use value <- result.try(dynamic.string(data))
  case value {
    "succeeded" -> Ok(Succeeded)
    "failed" -> Ok(Failed)
    "cancelled" -> Ok(Cancelled)
    _ -> invalid()
  }
}

fn status(data: Dynamic) -> Result(TaskStatus, DecodeErrors) {
  use value <- result.try(dynamic.string(data))
  case value {
    "succeeded" -> Ok(TaskSucceeded)
    "failed" -> Ok(TaskFailed)
    "skipped" -> Ok(TaskSkipped)
    "cancelled" -> Ok(TaskCancelled)
    "timed_out" -> Ok(TaskTimedOut)
    _ -> invalid()
  }
}

fn cache(data: Dynamic) -> Result(Cache, DecodeErrors) {
  use outcome <- result.try(dynamic.field("outcome", dynamic.string)(data))
  use key <- result.try(dynamic.optional_field("key", dynamic.string)(data))
  use reason <- result.try(dynamic.field("reason", dynamic.string)(data))
  Ok(Cache(outcome:, key:, reason:))
}

fn task(data: Dynamic) -> Result(TaskResult, DecodeErrors) {
  use id <- result.try(dynamic.field("id", dynamic.string)(data))
  use status <- result.try(dynamic.field("status", status)(data))
  use exit_code <- result.try(dynamic.optional_field("exit_code", dynamic.int)(
    data,
  ))
  use error <- result.try(dynamic.optional_field("error", dynamic.string)(data))
  use duration_ms <- result.try(dynamic.field("duration_ms", dynamic.int)(data))
  use stdout <- result.try(dynamic.field("stdout", dynamic.string)(data))
  use stderr <- result.try(dynamic.field("stderr", dynamic.string)(data))
  use cache <- result.try(dynamic.field("cache", cache)(data))
  case duration_ms >= 0 && id != "" {
    True ->
      Ok(TaskResult(
        id:,
        status:,
        exit_code:,
        error:,
        duration_ms:,
        stdout:,
        stderr:,
        cache:,
      ))
    False -> invalid()
  }
}

fn report(data: Dynamic) -> Result(RunResult, DecodeErrors) {
  use tasks <- result.try(dynamic.field("tasks", dynamic.list(task))(data))
  use outcome <- result.try(dynamic.optional_field("outcome", outcome)(data))
  let ids = dict.from_list(list.map(tasks, fn(task) { #(task.id, Nil) }))
  use _ <- result.try(case dict.size(ids) == list.length(tasks) {
    True -> Ok(Nil)
    False -> invalid()
  })
  let success = list.all(tasks, fn(task) { task.status == TaskSucceeded })
  case outcome, success {
    Some(Succeeded), False -> invalid()
    Some(Failed), True | Some(Cancelled), True -> invalid()
    _, _ -> Ok(RunResult(outcome:, tasks:))
  }
}

@external(javascript, "../../host_result_ffi.mjs", "parse_json")
fn parse_json(
  text: String,
  ok: fn(Dynamic) -> Result(RunResult, Nil),
  error: fn() -> Result(RunResult, Nil),
) -> Result(RunResult, Nil)

/// Reject malformed/unknown/nonterminal reports. Missing outcome supports older engines.
pub fn parse(text: String) -> Result(RunResult, Nil) {
  parse_json(
    text,
    fn(data) { report(data) |> result.map_error(fn(_) { Nil }) },
    fn() { Error(Nil) },
  )
}

pub fn succeeded(report: RunResult) -> Bool {
  list.all(report.tasks, fn(task) { task.status == TaskSucceeded })
}
