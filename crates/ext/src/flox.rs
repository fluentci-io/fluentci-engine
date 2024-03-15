use std::{
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{nix::Nix, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Flox {}

impl Extension for Flox {
    fn exec(
        &self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
    ) -> Result<ExitStatus, Error> {
        self.setup()?;

        let (stdout_tx, stdout_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (stderr_tx, stderr_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
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

    fn setup(&self) -> Result<(), Error> {
        Nix::default().setup()?;
        let status = Command::new("sh")
            .arg("-c")
            .arg("type flox > /dev/null")
            .spawn()?
            .wait()?;

        if status.success() {
            return Ok(());
        }

        Command::new("sh")
            .arg("-c")
            .arg("echo 'extra-trusted-substituters = https://cache.floxdev.com' | tee -a /etc/nix/nix.conf && echo 'extra-trusted-public-keys = flox-store-public-0:8c/B+kjIaQ+BloCmNkRUKwaVPFWkriSAd0JJvuDu4F0=' | tee -a /etc/nix/nix.conf")
            .spawn()?
            .wait()?;

        Command::new("sh")
            .arg("-c")
            .arg(
                "nix \
                profile \
                install \
                --impure \
                --experimental-features \
                nix-command flakes auto-allocate-uids \
                --accept-flake-config \
                github:flox/floxpkgs#flox.fromCatalog",
            )
            .spawn()?
            .wait()?;

        Ok(())
    }
}
