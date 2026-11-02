use fluentci_executor::{
    events::{Event, EventKind},
    Plan, Status,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{self, IsTerminal, Read, Seek, SeekFrom, Write},
    path::Path,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Plain,
    Tui,
    Json,
}
struct Row {
    status: Status,
    cache: String,
    duration_ms: u128,
    started_ms: Option<u128>,
}

/// One renderer for the whole execution; never one terminal per command.
pub struct Progress {
    mode: Mode,
    rows: BTreeMap<String, Row>,
    order: Vec<String>,
    tail: VecDeque<String>,
    offset: u64,
    pending: Vec<u8>,
    rendered_lines: usize,
    started: Instant,
    last_render: Instant,
    dirty: bool,
}
impl Progress {
    pub fn new(plan: &Plan, mode: &str) -> Result<Self, String> {
        let terminal =
            io::stdout().is_terminal() && std::env::var("TERM").unwrap_or_default() != "dumb";
        let mode =
            match mode {
                "auto" if terminal => Mode::Tui,
                "auto" | "plain" => Mode::Plain,
                "json" => Mode::Json,
                "tui" if terminal => Mode::Tui,
                "tui" => return Err(
                    "TUI progress requires an interactive terminal; use --progress plain or json"
                        .into(),
                ),
                _ => return Err("progress must be auto, plain, tui, or json".into()),
            };
        if mode == Mode::Tui {
            print!("\x1b[?25l");
            io::stdout().flush().map_err(|e| e.to_string())?;
        }
        Ok(Self {
            mode,
            rows: plan
                .tasks
                .iter()
                .map(|t| {
                    (
                        t.id.clone(),
                        Row {
                            status: Status::Pending,
                            cache: "disabled".into(),
                            duration_ms: 0,
                            started_ms: None,
                        },
                    )
                })
                .collect(),
            order: plan.tasks.iter().map(|t| t.id.clone()).collect(),
            tail: VecDeque::new(),
            offset: 0,
            pending: Vec::new(),
            rendered_lines: 0,
            started: Instant::now(),
            last_render: Instant::now() - Duration::from_secs(1),
            dirty: true,
        })
    }
    pub fn poll(&mut self, path: &Path, final_read: bool) -> Result<(), String> {
        match File::open(path) {
            Ok(mut file) => {
                file.seek(SeekFrom::Start(self.offset))
                    .map_err(|e| e.to_string())?;
                let mut buffer = [0u8; 65536];
                loop {
                    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    self.offset += n as u64;
                    self.pending.extend_from_slice(&buffer[..n]);
                    while let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
                        let line: Vec<_> = self.pending.drain(..=end).collect();
                        let event: Event = serde_json::from_slice(&line)
                            .map_err(|e| format!("invalid execution event: {e}"))?;
                        self.event(event)?;
                    }
                    // Bound work per UI tick; the journal remains lossless on disk.
                    if !final_read {
                        break;
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        if self.mode == Mode::Tui
            && (final_read || self.last_render.elapsed() >= Duration::from_millis(100))
        {
            self.render()?;
        }
        Ok(())
    }
    fn event(&mut self, event: Event) -> Result<(), String> {
        if self.mode == Mode::Json {
            serde_json::to_writer(io::stdout().lock(), &event).map_err(|e| e.to_string())?;
            println!();
            return Ok(());
        }
        match event.kind {
            EventKind::TaskState {
                id,
                status,
                cache,
                reason,
                error,
                duration_ms,
            } => {
                if let Some(row) = self.rows.get_mut(&id) {
                    if status == Status::Running && row.started_ms.is_none() {
                        row.started_ms = Some(u128::from(event.elapsed_ms));
                    }
                    row.status = status.clone();
                    row.cache = cache.clone();
                    row.duration_ms = u128::from(duration_ms);
                }
                if self.mode == Mode::Plain {
                    println!(
                        "[{}] {:?} cache={} {}",
                        clean(&id),
                        status,
                        cache,
                        clean(error.as_deref().unwrap_or(&reason))
                    );
                } else if let Some(error) = error {
                    self.push_log(format!("{}: {}", id, error));
                }
            }
            EventKind::Log { id, stream, text } => {
                if self.mode == Mode::Plain {
                    for line in text.split_inclusive('\n') {
                        print!("[{}:{}] {}", clean(&id), stream, clean(line));
                    }
                    io::stdout().flush().map_err(|e| e.to_string())?;
                } else {
                    for line in text.lines() {
                        self.push_log(format!("{id}:{stream} {line}"));
                    }
                }
            }
            EventKind::RunStarted { tasks } => {
                if self.mode == Mode::Plain {
                    println!("Running {tasks} tasks");
                }
            }
            EventKind::RunFinished { success } => {
                if self.mode == Mode::Plain {
                    println!(
                        "Pipeline {}",
                        if success {
                            "succeeded"
                        } else {
                            "failed or cancelled"
                        }
                    );
                }
            }
        }
        self.dirty = true;
        Ok(())
    }
    fn push_log(&mut self, line: String) {
        self.tail
            .push_back(clean(&line).chars().take(4096).collect());
        while self.tail.len() > 8 {
            self.tail.pop_front();
        }
    }
    fn render(&mut self) -> Result<(), String> {
        let running = self.rows.values().any(|r| r.status == Status::Running);
        if !self.dirty && !running {
            return Ok(());
        }
        let (width, height) = crossterm::terminal::size().unwrap_or((100, 30));
        let width = usize::from(width.saturating_sub(1)).max(1);
        let max_rows = usize::from(height.saturating_sub(12)).max(1);
        let elapsed = self.started.elapsed().as_millis();
        let spinner = ['|', '/', '-', '\\'][(elapsed / 100 % 4) as usize];
        let complete = self
            .rows
            .values()
            .filter(|r| r.status == Status::Succeeded)
            .count();
        let hits = self.rows.values().filter(|r| r.cache == "hit").count();
        let mut lines = vec![format!(
            "FluentCI  {complete}/{} succeeded  {hits} cached  {:.1}s",
            self.rows.len(),
            elapsed as f64 / 1000.0
        )];
        // Keep active tasks visible even when the plan is larger than the terminal.
        let mut ids: Vec<_> = self.order.iter().collect();
        ids.sort_by_key(|id| self.rows[*id].status != Status::Running);
        for id in ids.iter().take(max_rows) {
            let row = &self.rows[*id];
            let icon = match row.status {
                Status::Running => spinner,
                Status::Succeeded => '+',
                Status::Pending => '.',
                Status::Skipped => '-',
                _ => '!',
            };
            let duration = if row.status == Status::Running {
                elapsed.saturating_sub(row.started_ms.unwrap_or(elapsed))
            } else {
                row.duration_ms
            };
            lines.push(format!(
                " {icon} {:<24} {:<10?} {:>6.1}s {}",
                clean(id),
                row.status,
                duration as f64 / 1000.0,
                if row.cache == "hit" { "CACHED" } else { "" }
            ));
        }
        if ids.len() > max_rows {
            lines.push(format!(
                " ... {} more tasks (all results retained in result.json)",
                ids.len() - max_rows
            ));
        }
        lines.push("Recent output:".into());
        lines.extend(self.tail.iter().cloned());
        lines.truncate(usize::from(height.saturating_sub(1)).max(1));
        let mut out = io::stdout().lock();
        if self.rendered_lines > 0 {
            write!(out, "\x1b[{}A", self.rendered_lines).map_err(|e| e.to_string())?;
        }
        write!(out, "\x1b[J").map_err(|e| e.to_string())?;
        for line in &lines {
            writeln!(out, "{}", line.chars().take(width).collect::<String>())
                .map_err(|e| e.to_string())?;
        }
        out.flush().map_err(|e| e.to_string())?;
        self.rendered_lines = lines.len();
        self.last_render = Instant::now();
        self.dirty = false;
        Ok(())
    }
}
impl Drop for Progress {
    fn drop(&mut self) {
        if self.mode == Mode::Tui {
            print!("\x1b[?25h");
            let _ = io::stdout().flush();
        }
    }
}
fn clean(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_control_sequences_cannot_be_injected_by_task_output() {
        assert_eq!(clean("hello\x1b[2J\r\x08world\n"), "hello[2Jworld\n");
    }
}
