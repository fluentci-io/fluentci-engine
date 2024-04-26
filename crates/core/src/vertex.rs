use std::{
    process::ExitStatus,
    sync::{mpsc::Sender, Arc},
};

use anyhow::Error;
use fluentci_ext::Extension;
use fluentci_types::Output;
use owo_colors::OwoColorize;

#[derive(Debug, Clone, Default)]
pub struct VertexExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

pub trait Runnable {
    fn run(
        &mut self,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error>;
}

#[derive(Clone)]
pub struct Vertex {
    pub id: String,
    pub label: String,
    pub command: String,
    pub needs: Vec<String>,
    pub runner: Arc<Box<dyn Extension + Send + Sync>>,
}

impl Runnable for Vertex {
    fn run(
        &mut self,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error> {
        let label = format!("[{}]", self.label);
        println!("{} {}", label.cyan(), self.id.bright_yellow());
        println!("{} {}", label.cyan(), self.command.bright_green());
        fluentci_logging::info(&format!("{} {}", label, self.id), "fluentci-core")?;
        fluentci_logging::info(&format!("{} {}", label, self.command), "fluentci-core")?;

        if let Some(runner) = Arc::get_mut(&mut self.runner) {
            runner.exec(&self.command, tx, out, last_cmd, work_dir)
        } else {
            Err(Error::msg("Failed to obtain mutable reference to runner"))
        }
    }
}
