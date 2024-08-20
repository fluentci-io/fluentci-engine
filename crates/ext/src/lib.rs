use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Instant,
};

use anyhow::Error;
use fluentci_types::Output;
use owo_colors::OwoColorize;
use superconsole::{
    style::Stylize, Component, Dimensions, DrawMode, Line, Lines, Span, SuperConsole,
};

pub mod archive;
pub mod cache;
pub mod devbox;
pub mod devenv;
pub mod envhub;
pub mod flox;
pub mod git;
pub mod git_checkout;
pub mod git_last_commit;
pub mod hash;
pub mod hermit;
pub mod http;
pub mod mise;
pub mod nix;
pub mod pixi;
pub mod pkgx;
pub mod proto;
pub mod runner;
pub mod service;

pub trait Extension {
    fn exec(
        &mut self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error>;
    fn setup(&self) -> Result<(), Error>;
    fn post_setup(&self, tx: Sender<String>) -> Result<ExitStatus, Error> {
        tx.send("".into())?;
        Ok(ExitStatus::default())
    }
    fn format_command(&self, cmd: &str) -> String {
        format!("{}", cmd)
    }
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
    let namespace = cmd.to_string().clone();

    thread::spawn(move || {
        if let Some(mut console) = SuperConsole::new() {
            if env::var("FLUENTCI_CONSOLE").unwrap_or_default() == "0" {
                let mut stdout = String::new();
                while let Ok(line) = stdout_rx.recv() {
                    println!("{}", line);
                    match fluentci_logging::info(&line, &namespace) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                    stdout.push_str(&line);
                    stdout.push_str("\n");
                }

                if out_clone == Output::Stdout && last_cmd {
                    match tx_clone.send(stdout) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                return;
            }
            let created = Instant::now();
            let mut stdout = String::new();
            let mut lines = vec![];
            while let Ok(line) = stdout_rx.recv() {
                lines.push(line.clone());
                if lines.len() > 15 {
                    lines.remove(0);
                }

                console
                    .render(&Log {
                        command: &namespace,
                        lines: lines.clone(),
                        created,
                        now: Instant::now(),
                    })
                    .unwrap();
                match fluentci_logging::info(&line, &namespace) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                stdout.push_str(&line);
                stdout.push_str("\n");
            }

            console
                .finalize(&Log {
                    command: &namespace,
                    lines: vec![],
                    created,
                    now: Instant::now(),
                })
                .unwrap();
            if out_clone == Output::Stdout && last_cmd {
                match tx_clone.send(stdout) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            return;
        }

        let mut stdout = String::new();
        while let Ok(line) = stdout_rx.recv() {
            println!("{}", line);
            match fluentci_logging::info(&line, &namespace) {
                Ok(_) => {}
                Err(_) => {}
            }
            stdout.push_str(&line);
            stdout.push_str("\n");
        }

        if out_clone == Output::Stdout && last_cmd {
            match tx_clone.send(stdout) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    });

    let namespace = cmd.to_string().clone();

    thread::spawn(move || {
        let mut stderr = String::new();
        while let Ok(line) = stderr_rx.recv() {
            println!("{}", line);
            match fluentci_logging::info(&line, &namespace) {
                Ok(_) => {}
                Err(_) => {}
            }
            stderr.push_str(&line);
            stderr.push_str("\n");
        }
        if out == Output::Stderr && last_cmd {
            match tx.send(stderr) {
                Ok(_) => {}
                Err(_) => {}
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

struct Log<'a> {
    command: &'a str,
    lines: Vec<String>,
    created: Instant,
    now: Instant,
}

impl<'a> Component for Log<'a> {
    fn draw_unchecked(&self, _dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        Ok(match mode {
            DrawMode::Normal => {
                let elapsed = self.now.duration_since(self.created).as_millis();
                let mut lines: Vec<Line> = self
                    .lines
                    .iter()
                    .map(|l| vec![l.clone()].try_into().unwrap())
                    .collect();
                lines.push(
                    vec![format!(
                        "Executing {} {}{}",
                        self.command.bright_purple(),
                        (elapsed as f64 / 1000.0).to_string().magenta(),
                        "s".magenta()
                    )]
                    .try_into()
                    .unwrap(),
                );
                Lines(lines)
            }
            DrawMode::Final => {
                const FINISHED: &str = "   Finished ";
                let finished = Span::new_styled(FINISHED.to_owned().green().bold())?;
                let completion = format!(
                    "{} in {}{}",
                    self.command.bright_purple(),
                    (self.now.duration_since(self.created).as_millis() as f64 / 1000.0)
                        .to_string()
                        .magenta(),
                    "s".magenta()
                );
                Lines(vec![Line::from_iter([
                    finished,
                    Span::new_unstyled(&completion)?,
                ])])
            }
        })
    }
}
