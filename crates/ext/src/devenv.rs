use std::{
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, nix::Nix, pkgx::Pkgx, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Devenv {}

impl Extension for Devenv {
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

        Pkgx::default().install(vec!["direnv"])?;

        Command::new("bash")
            .arg("-c")
            .arg("[ -f devenv.nix ] || devenv init")
            .current_dir(work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;

        let cmd = format!("devenv shell {}", cmd);
        exec(&cmd, tx, out, last_cmd, work_dir)
    }

    fn setup(&self) -> Result<(), Error> {
        Nix::default().setup()?;

        let mut child = Command::new("sh")
            .arg("-c")
            .arg("type devenv > /dev/null")
            .spawn()?;
        let status = child.wait()?;

        if status.success() {
            return Ok(());
        }

        Command::new("sh")
            .arg("-c")
            .arg(r#"echo "trusted-users = root $USER" | tee -a /etc/nix/nix.conf"#)
            .spawn()?
            .wait()?;

        Command::new("sh")
            .arg("-c")
            .arg("nix profile install --accept-flake-config github:cachix/cachix")
            .spawn()?
            .wait()?;

        Command::new("sh")
            .arg("-c")
            .arg("cachix use devenv")
            .spawn()?
            .wait()?;

        Command::new("sh")
            .arg("-c")
            .arg("nix profile install --accept-flake-config tarball+https://install.devenv.sh/latest")
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn format_command(&self, cmd: &str) -> String {
        format!("[ -f devenv.nix ] || devenv init ; devenv shell {}", cmd)
    }
}
