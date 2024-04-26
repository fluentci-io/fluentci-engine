use std::{
    env,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, Extension};
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
        &mut self,
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

        let cmd = format!("eval \"$(pkgx --shellcode)\" && dev ; {}", cmd);
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
            .arg("type pkgx > /dev/null || curl -fsS https://pkgx.sh | sh")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait()?;

        Ok(())
    }

    fn format_command(&self, cmd: &str) -> String {
        format!("eval \"$(pkgx --shellcode)\" && dev ; {}", cmd)
    }
}
