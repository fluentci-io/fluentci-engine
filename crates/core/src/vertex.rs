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
        &self,
        runner: Arc<Box<dyn Extension + Send + Sync>>,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error>;
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: String,
    pub label: String,
    pub command: String,
    pub needs: Vec<String>,
}

impl Runnable for Vertex {
    fn run(
        &self,
        runner: Arc<Box<dyn Extension + Send + Sync>>,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error> {
        let label = format!("[{}]", self.label);
        println!("{} {}", label.cyan(), self.id.bright_yellow());
        println!("{} {}", label.cyan(), self.command.bright_green());

        runner.exec(&self.command, tx, out, last_cmd, work_dir)
    }
}
