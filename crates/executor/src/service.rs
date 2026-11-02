//! Readiness and health probes run as supervised children, never in the scheduler thread.
use crate::{Running, Status, Task};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub readiness: Probe,
    /// Optional recurring check after readiness; defaults to the readiness probe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health: Option<Probe>,
    #[serde(default = "startup_timeout")]
    pub startup_timeout_ms: u64,
    #[serde(default = "shutdown_timeout")]
    pub shutdown_timeout_ms: u64,
    #[serde(default = "failure_threshold")]
    pub failure_threshold: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "probe_interval")]
    pub interval_ms: u64,
    #[serde(default = "probe_timeout")]
    pub timeout_ms: u64,
}
fn startup_timeout() -> u64 {
    30_000
}
fn shutdown_timeout() -> u64 {
    5_000
}
fn failure_threshold() -> usize {
    3
}
fn probe_interval() -> u64 {
    250
}
fn probe_timeout() -> u64 {
    1_000
}
impl Service {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.readiness.program.is_empty()
            || self.startup_timeout_ms == 0
            || self.shutdown_timeout_ms == 0
            || self.readiness.interval_ms == 0
            || self.readiness.timeout_ms == 0
            || self.health.as_ref().is_some_and(|probe| {
                probe.program.is_empty() || probe.interval_ms == 0 || probe.timeout_ms == 0
            })
            || self.failure_threshold == 0
        {
            return Err("service requires a probe and positive lifecycle limits".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProbeLogs {
    pub stdout: PathBuf,
    pub stderr: PathBuf,
}

pub(crate) enum Health {
    Waiting,
    Ready,
    Failed(String),
}
pub(crate) struct Supervisor {
    probe: Option<Running>,
    next: Instant,
    failures: usize,
    pub ready: bool,
    logs: ProbeLogs,
}
impl Supervisor {
    pub fn new(logs: ProbeLogs) -> Self {
        Self {
            probe: None,
            next: Instant::now(),
            failures: 0,
            ready: false,
            logs,
        }
    }
    pub fn stop(&mut self) {
        self.probe.take();
    }
    pub fn poll(&mut self, task: &Task, inherited: &BTreeMap<OsString, OsString>) -> Health {
        let config = task.service.as_ref().expect("service configuration");
        let check = if self.ready {
            config.health.as_ref().unwrap_or(&config.readiness)
        } else {
            &config.readiness
        };
        if let Some(probe) = &mut self.probe {
            let outcome = match probe.child.try_wait() {
                Ok(Some(status)) => Some(status.success()),
                Ok(None) if probe.started.elapsed() < Duration::from_millis(check.timeout_ms) => {
                    None
                }
                _ => Some(false),
            };
            if let Some(success) = outcome {
                self.probe.take();
                let next_check = if success {
                    config.health.as_ref().unwrap_or(&config.readiness)
                } else {
                    check
                };
                self.next = Instant::now() + Duration::from_millis(next_check.interval_ms);
                if success {
                    self.failures = 0;
                    if !self.ready {
                        self.ready = true;
                        return Health::Ready;
                    }
                } else {
                    self.failures += 1;
                    if self.ready && self.failures >= config.failure_threshold {
                        return Health::Failed(
                            "service health probe failure threshold reached".into(),
                        );
                    }
                }
            }
        } else if Instant::now() >= self.next {
            let phase = if self.ready { "health" } else { "readiness" };
            let open = |path: &PathBuf| -> Result<_, std::io::Error> {
                let mut file = OpenOptions::new().create(true).append(true).open(path)?;
                writeln!(file, "--- {phase} probe ---")?;
                Ok(file)
            };
            let (stdout, stderr) = match (open(&self.logs.stdout), open(&self.logs.stderr)) {
                (Ok(stdout), Ok(stderr)) => (stdout, stderr),
                _ => return Health::Failed("could not open service probe logs".into()),
            };
            let mut command = Command::new(&check.program);
            command
                .args(&check.args)
                .current_dir(&task.cwd)
                .env_clear()
                .envs(inherited)
                .envs(&task.env)
                .stdin(Stdio::null())
                .stdout(Stdio::from(stdout))
                .stderr(Stdio::from(stderr));
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
            match command.spawn() {
                Ok(child) => {
                    self.probe = Some(Running {
                        index: usize::MAX,
                        child,
                        started: Instant::now(),
                        leases: None,
                        action: None,
                    })
                }
                Err(_) => return Health::Failed("could not spawn service health probe".into()),
            }
        }
        Health::Waiting
    }
}

pub(crate) struct Shutdown {
    pub started: Option<Instant>,
    pub status: Status,
    pub error: Option<String>,
}
