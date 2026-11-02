//! Readiness and health probes run as supervised children, never in the scheduler thread.
use crate::{Running, Status, Task};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub readiness: Probe,
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
            || self.failure_threshold == 0
        {
            return Err("service requires a probe and positive lifecycle limits".into());
        }
        Ok(())
    }
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
}
impl Supervisor {
    pub fn new() -> Self {
        Self {
            probe: None,
            next: Instant::now(),
            failures: 0,
            ready: false,
        }
    }
    pub fn stop(&mut self) {
        self.probe.take();
    }
    pub fn poll(&mut self, task: &Task, inherited: &BTreeMap<OsString, OsString>) -> Health {
        let config = task.service.as_ref().expect("service configuration");
        if let Some(probe) = &mut self.probe {
            let outcome = match probe.child.try_wait() {
                Ok(Some(status)) => Some(status.success()),
                Ok(None)
                    if probe.started.elapsed()
                        < Duration::from_millis(config.readiness.timeout_ms) =>
                {
                    None
                }
                _ => Some(false),
            };
            if let Some(success) = outcome {
                self.probe.take();
                self.next = Instant::now() + Duration::from_millis(config.readiness.interval_ms);
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
            let mut command = Command::new(&config.readiness.program);
            command
                .args(&config.readiness.args)
                .current_dir(&task.cwd)
                .env_clear()
                .envs(inherited)
                .envs(&task.env)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
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
