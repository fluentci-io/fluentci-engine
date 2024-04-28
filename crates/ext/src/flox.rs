use std::{
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, nix::Nix, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Flox {}

impl Extension for Flox {
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

        Command::new("bash")
            .arg("-c")
            .arg("[ -d .flox ] || flox init")
            .current_dir(work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;

        let cmd = format!("flox activate -- {}", cmd);
        exec(&cmd, tx, out, last_cmd, work_dir)
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

        let sudo = Command::new("sh")
            .arg("-c")
            .arg("type sudo > /dev/null")
            .spawn()?
            .wait()?;

        let sudo = if sudo.success() { "sudo" } else { "" };

        Command::new("sh")
            .arg("-c")
            .arg(&format!("echo 'extra-trusted-substituters = https://cache.floxdev.com' | {} tee -a /etc/nix/nix.conf && echo 'extra-trusted-public-keys = flox-cache-public-1:7F4OyH7ZCnFhcze3fJdfyXYLQw/aV7GEed86nQ7IsOs=' | {} tee -a /etc/nix/nix.conf", sudo, sudo))
            .spawn()?
            .wait()?;

        Command::new("sh")
            .arg("-c")
            .arg(
                "nix profile install --impure \
                --experimental-features 'nix-command flakes' \
                --accept-flake-config \
                github:flox/flox",
            )
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn format_command(&self, cmd: &str) -> String {
        format!("[ -d .flox ] || flox init ; flox activate -- {}", cmd)
    }
}
