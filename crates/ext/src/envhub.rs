use std::{
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, nix::Nix, pkgx::Pkgx, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Envhub {}

impl Extension for Envhub {
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

        exec(cmd, tx, out, last_cmd, work_dir)
    }

    fn setup(&self) -> Result<(), Error> {
        Nix::default().setup()?;
        Pkgx::default().install(vec!["curl", "git"])?;
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("curl -sSL https://install.envhub.sh | bash")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait().map_err(Error::from)?;
        Ok(())
    }
}
