import fluentci/host/journal
import gleam/int
import gleam/list
import gleam/option.{Some}
import gleeunit/should

fn record(sequence: Int, fields: String) -> String {
  "{\"sequence\":"
  <> int.to_string(sequence)
  <> ",\"elapsed_ms\":0,"
  <> fields
  <> "}\n"
}

pub fn typed_journal_variants_test() {
  let text =
    record(0, "\"type\":\"run_started\",\"tasks\":1")
    <> record(
      1,
      "\"type\":\"task_state\",\"id\":\"db\",\"status\":\"ready\",\"cache\":\"disabled\",\"reason\":\"service\",\"wait_reason\":\"readiness\",\"error\":null,\"duration_ms\":10",
    )
    <> record(
      2,
      "\"type\":\"log\",\"id\":\"db\",\"stream\":\"probe_stderr\",\"text\":\"diagnostic\\n\",\"trace_id\":\"trace\",\"span_id\":\"span\"",
    )
    <> record(3, "\"type\":\"run_finished\",\"success\":true")
  let assert Ok([started, state, log, finished]) = journal.parse(text)
  started.kind |> should.equal(journal.RunStarted(1))
  let assert journal.TaskState(
    "db",
    journal.Ready,
    "disabled",
    "service",
    Some("readiness"),
    _,
    10,
  ) = state.kind
  log.kind
  |> should.equal(journal.Log("db", journal.ProbeStderr, "diagnostic\n"))
  log.trace_id |> should.equal(Some("trace"))
  log.span_id |> should.equal(Some("span"))
  finished.kind |> should.equal(journal.RunFinished(True))
  journal.parse("") |> should.equal(Ok([]))
}

pub fn reject_invalid_journal_test() {
  let valid = record(2, "\"type\":\"run_finished\",\"success\":true")
  journal.parse(valid <> valid) |> should.equal(Error(Nil))
  journal.parse(valid <> record(1, "\"type\":\"run_started\",\"tasks\":1"))
  |> should.equal(Error(Nil))
  list.each(
    [
      "{}",
      "{}\n",
      "invalid\n",
      "\n",
      record(-1, "\"type\":\"run_started\",\"tasks\":1"),
      record(0, "\"type\":\"unknown\""),
      record(0, "\"type\":\"run_finished\",\"success\":\"true\""),
      record(
        0,
        "\"type\":\"log\",\"id\":\"a\",\"stream\":\"unknown\",\"text\":\"hi\"",
      ),
      record(
        0,
        "\"type\":\"log\",\"id\":\"\",\"stream\":\"stdout\",\"text\":\"hi\"",
      ),
      record(
        0,
        "\"type\":\"task_state\",\"id\":\"a\",\"status\":\"unknown\",\"cache\":\"hit\",\"reason\":\"cache\",\"duration_ms\":0",
      ),
    ],
    fn(text) { journal.parse(text) |> should.equal(Error(Nil)) },
  )
}

pub fn lifecycle_statuses_test() {
  list.each(
    [
      "pending",
      "running",
      "ready",
      "stopping",
      "succeeded",
      "failed",
      "skipped",
      "cancelled",
      "timed_out",
    ],
    fn(status) {
      let assert Ok([_]) =
        journal.parse(record(
          0,
          "\"type\":\"task_state\",\"id\":\"a\",\"status\":\""
            <> status
            <> "\",\"cache\":\"disabled\",\"reason\":\"none\",\"duration_ms\":0",
        ))
      Nil
    },
  )
}
