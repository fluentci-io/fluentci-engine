use std::{
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use anyhow::Error;
use fluentci_types::Output;

pub mod devbox;
pub mod devenv;
pub mod envhub;
pub mod flox;
pub mod git;
pub mod git_checkout;
pub mod git_last_commit;
pub mod http;
pub mod mise;
pub mod nix;
pub mod pixi;
pub mod pkgx;
pub mod runner;

pub trait Extension {
    fn exec(
        &self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error>;
    fn setup(&self) -> Result<(), Error>;
}

pub fn exec(
    cmd: &str,
    tx: Sender<String>,
    out: Output,
    last_cmd: bool,
    work_dir: &str,
) -> Result<ExitStatus, Error> {
    let (stdout_tx, stdout_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (stderr_tx, stderr_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .current_dir(work_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

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
            tx_clone.send(stdout).unwrap();
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
            tx.send(stderr).unwrap();
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

    child.wait().map_err(Error::from)
}
