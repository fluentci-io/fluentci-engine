use std::{
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, nix::Nix, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Devbox {}

impl Devbox {
    pub fn install(&self, pkgs: Vec<&str>) -> Result<ExitStatus, Error> {
        self.setup()?;
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(format!("devbox global add {}", pkgs.join(" ")))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait().map_err(Error::from)
    }
}

impl Extension for Devbox {
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

        Command::new("bash")
            .arg("-c")
            .arg("[ -f devbox.json ] || devbox init")
            .current_dir(work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;

        let cmd = format!("devbox run {}", cmd);
        exec(&cmd, tx, out, last_cmd, work_dir)
    }

    fn setup(&self) -> Result<(), Error> {
        Nix::default().setup()?;
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("type devbox > /dev/null || curl -fsSL https://get.jetpack.io/devbox | bash")
            .env("FORCE", "1")
            .spawn()?;
        child.wait()?;
        Ok(())
    }
}
