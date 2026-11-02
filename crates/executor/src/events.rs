//! Durable, ordered execution events. UI consumers tail this journal rather than
//! placing terminal rendering or network backpressure on the scheduler.
use crate::{telemetry::Telemetry, Status, Task, TaskResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
    time::Instant,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    pub sequence: u64,
    pub elapsed_ms: u64,
    #[serde(flatten)]
    pub kind: EventKind,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    RunStarted {
        tasks: usize,
    },
    TaskState {
        id: String,
        status: Status,
        cache: String,
        reason: String,
        #[serde(default)]
        wait_reason: Option<String>,
        error: Option<String>,
        duration_ms: u64,
    },
    Log {
        id: String,
        stream: String,
        text: String,
    },
    RunFinished {
        success: bool,
    },
}

pub(crate) struct Journal {
    telemetry: Telemetry,
    task_ids: HashMap<String, usize>,
    writer: BufWriter<File>,
    started: Instant,
    sequence: u64,
    states: Vec<(Status, String, Option<String>)>,
    offsets: Vec<[u64; 4]>,
}
impl Journal {
    pub fn new(path: &Path, tasks: &[Task]) -> Result<Self, String> {
        let count = tasks.len();
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let mut journal = Self {
            telemetry: Telemetry::new(tasks),
            task_ids: tasks
                .iter()
                .enumerate()
                .map(|(i, t)| (t.id.clone(), i))
                .collect(),
            writer: BufWriter::new(file),
            started: Instant::now(),
            sequence: 0,
            states: vec![(Status::Pending, String::new(), None); count],
            offsets: vec![[0; 4]; count],
        };
        journal.emit(EventKind::RunStarted { tasks: count })?;
        journal.writer.flush().map_err(|e| e.to_string())?;
        Ok(journal)
    }
    fn emit(&mut self, kind: EventKind) -> Result<(), String> {
        let index = match &kind {
            EventKind::TaskState { id, .. } | EventKind::Log { id, .. } => {
                self.task_ids.get(id).copied()
            }
            _ => None,
        };
        let (trace_id, span_id) = self.telemetry.ids(index);
        let event = Event {
            trace_id,
            span_id,
            sequence: self.sequence,
            elapsed_ms: self.started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            kind,
        };
        self.sequence += 1;
        serde_json::to_writer(&mut self.writer, &event).map_err(|e| e.to_string())?;
        self.writer.write_all(b"\n").map_err(|e| e.to_string())
    }
    pub fn update(
        &mut self,
        results: &[TaskResult],
        order: &[usize],
        drain: bool,
    ) -> Result<(), String> {
        for &i in order {
            let result = &results[i];
            for (stream, path) in [
                Some(("stdout", &result.stdout)),
                Some(("stderr", &result.stderr)),
                result
                    .probe_logs
                    .as_ref()
                    .map(|logs| ("probe_stdout", &logs.stdout)),
                result
                    .probe_logs
                    .as_ref()
                    .map(|logs| ("probe_stderr", &logs.stderr)),
            ]
            .into_iter()
            .enumerate()
            {
                let Some(path) = path else { continue };
                // A rejected restore can replace/remove these files before spawn.
                // Do not emit tentative logs or advance their cursors.
                if result.cache.outcome == "preparing" {
                    continue;
                }
                let mut file = match File::open(path.1) {
                    Ok(f) => f,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(e.to_string()),
                };
                file.seek(SeekFrom::Start(self.offsets[i][stream]))
                    .map_err(|e| e.to_string())?;
                // At most 64 KiB per stream per tick; the original log files retain
                // arbitrary binary output. Event text is a lossy UTF-8 display view.
                let mut buffer = [0u8; 16384];
                let mut chunks = 0;
                loop {
                    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    self.offsets[i][stream] += n as u64;
                    self.emit(EventKind::Log {
                        id: result.id.clone(),
                        stream: path.0.into(),
                        text: String::from_utf8_lossy(&buffer[..n]).into_owned(),
                    })?;
                    chunks += 1;
                    if !drain && chunks >= 4 {
                        break;
                    }
                }
            }
            let state = (
                result.status.clone(),
                result.cache.outcome.clone(),
                result.wait_reason.clone(),
            );
            if state != self.states[i] {
                self.telemetry.update(i, result);
                self.states[i] = state;
                self.emit(EventKind::TaskState {
                    id: result.id.clone(),
                    status: result.status.clone(),
                    cache: result.cache.outcome.clone(),
                    reason: result.cache.reason.clone(),
                    wait_reason: result.wait_reason.clone(),
                    error: result.error.clone(),
                    duration_ms: result.duration_ms.min(u64::MAX as u128) as u64,
                })?;
            }
        }
        self.writer.flush().map_err(|e| e.to_string())
    }
    pub fn finish(&mut self, success: bool) -> Result<(), String> {
        self.telemetry.finish(success);
        self.emit(EventKind::RunFinished { success })?;
        self.writer.flush().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_event_roundtrips_through_the_wire_format() {
        for kind in [
            EventKind::RunStarted { tasks: 2 },
            EventKind::TaskState {
                id: "task".into(),
                status: Status::Running,
                cache: "miss".into(),
                reason: "new action".into(),
                wait_reason: None,
                error: None,
                duration_ms: 12,
            },
            EventKind::Log {
                id: "task".into(),
                stream: "stdout".into(),
                text: "hello\n".into(),
            },
            EventKind::RunFinished { success: true },
        ] {
            let encoded = serde_json::to_string(&Event {
                trace_id: None,
                span_id: None,
                sequence: 1,
                elapsed_ms: 42,
                kind,
            })
            .unwrap();
            let decoded: Event = serde_json::from_str(&encoded).unwrap();
            assert_eq!(serde_json::to_string(&decoded).unwrap(), encoded);
        }
    }
}
