use std::{
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use owo_colors::OwoColorize;

use crate::deps::Output;

#[derive(Debug, Clone, Default)]
pub struct VertexExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

pub trait Runnable {
    fn run(&self, tx: Sender<(String, ExitStatus)>, out: Output, last_cmd: bool) -> ExitStatus;
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: String,
    pub label: String,
    pub command: String,
    pub needs: Vec<String>,
}

impl Runnable for Vertex {
    fn run(&self, tx: Sender<(String, ExitStatus)>, out: Output, last_cmd: bool) -> ExitStatus {
        let label = format!("[{}]", self.label);
        println!("{} Execute: {}", label.cyan(), self.id.bright_yellow());
        let label = format!("[{}]", self.label);
        println!("{} Command: {}", label.cyan(), self.command.bright_green());

        if self.command.is_empty() {
            return ExitStatus::default();
        }

        let (stdout_tx, stdout_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (stderr_tx, stderr_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to execute command");

        let stdout_tx_clone = stdout_tx.clone();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let out_clone = out.clone();
        let tx_clone = tx.clone();

        thread::spawn(move || {
            let mut stdout = String::new();
            while let Ok(line) = stdout_rx.recv() {
                println!("{}", line);
                stdout.push_str(&line);
                stdout.push_str("\n");
            }
            if out_clone == Output::Stdout && last_cmd {
                tx_clone.send((stdout, ExitStatus::default())).unwrap();
            }
        });

        thread::spawn(move || {
            let mut stderr = String::new();
            while let Ok(line) = stderr_rx.recv() {
                println!("{}", line);
                stderr.push_str(&line);
                stderr.push_str("\n");
            }
            if out == Output::Stderr && last_cmd {
                tx.send((stderr, ExitStatus::default())).unwrap();
            }
        });

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                stdout_tx_clone.send(line.unwrap()).unwrap();
            }
        });

        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                stderr_tx.send(line.unwrap()).unwrap();
            }
        });

        child.wait().expect("Failed to wait on child")
    }
}
