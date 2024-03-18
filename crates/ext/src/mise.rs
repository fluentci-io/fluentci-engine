use std::{
    env,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Mise {}

impl Extension for Mise {
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

        let mut child = Command::new("sh")
            .arg("-c")
            .arg("mise install")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait()?;

        let cmd = format!("eval \"$(~/.local/bin/mise activate bash)\" && {}", cmd);
        exec(&cmd, tx, out, last_cmd, work_dir)
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
            .arg("type mise > /dev/null || curl https://mise.run | sh")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait()?;

        Ok(())
    }
}
