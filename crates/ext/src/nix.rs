use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
use users::get_current_username;

use crate::Extension;
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Nix {}

impl Extension for Nix {
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

        Command::new("bash")
            .arg("-c")
            .arg("nix flake init")
            .current_dir(work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(format!("nix develop -c {}", cmd))
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

    fn setup(&self) -> Result<(), Error> {
        let user = match get_current_username() {
            Some(user) => user.to_string_lossy().to_string(),
            None => "root".to_string(),
        };

        env::set_var("USER", user);
        env::set_var("SHELL", "/bin/bash");
        
        let home = match env::var("HOME") {
            Ok(home) => home,
            Err(_) => "/root".to_string(),
        };
        let nix_path = format!("{}/.nix-profile/bin", home);
        env::set_var(
            "PATH",
            format!(
                "{}:{}:{}",
                env::var("PATH")?,
                "/nix/var/nix/profiles/default/bin",
                nix_path
            ),
        );

        let mut child = Command::new("sh")
            .arg("-c")
            .arg("type systemctl > /dev/null")
            .spawn()?;
        let status = child.wait()?;
        let init = match status.code() {
            Some(0) => "",
            _ => "--init none",
        };

        let linux = match std::env::consts::OS {
            "linux" => format!("linux --extra-conf 'sandbox = false' {}", init),
            _ => "".to_string(),
        };
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(format!("type nix > /dev/null || curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install {}", linux))
            
            .spawn()?;
        child.wait()?;

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(format!("type nix > /dev/null || curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install {} --no-confirm", linux))
            
            .spawn()?;
        child.wait()?;
        Ok(())
    }
}
