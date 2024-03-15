use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::Extension;
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Pkgx {}

impl Pkgx {
    pub fn install(&self, pkgs: Vec<&str>) -> Result<ExitStatus, Error> {
        self.setup()?;

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(format!("pkgx install {}", pkgs.join(" ")))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait().map_err(Error::from)
    }
}

impl Extension for Pkgx {
    fn exec(
        &self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error> {
        self.setup()?;

        if cmd.is_empty() {
            return Ok(ExitStatus::default());
        }

        let (stdout_tx, stdout_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (stderr_tx, stderr_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(format!("eval \"$(pkgx --shellcode)\" && {}", cmd))
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
                match tx_clone.send(stdout) {
                    Ok(_) => {}
                    Err(e) => eprintln!("{}", e),
                }
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
                match tx.send(stderr) {
                    Ok(_) => {}
                    Err(e) => eprintln!("{}", e),
                }
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
        let path = format!(
            "{}:{}",
            env::var("PATH")?,
            format!("{}/.local/bin", env::var("HOME")?)
        );
        env::set_var("PATH", &path);
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("type pkgx > /dev/null || curl -fsS https://pkgx.sh | sh")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait()?;
        Ok(())
    }
}
