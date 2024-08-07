use std::{
    env,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};
use users::get_current_username;

use crate::{exec, Extension};
use anyhow::Error;
use fluentci_types::{Output, nix::NixArgs};

#[derive(Default, Clone)]
pub struct Nix {
    pub args: NixArgs
}

impl Nix {
    pub fn new(args: NixArgs) -> Self {
        Self { args }
    }

    pub fn impure(&mut self) -> &mut Self {
        self.args.impure = true;
        self
    }

    pub fn build_args(&self) -> String {
        let mut args = vec![];
        if self.args.impure {
            args.push("--impure".to_string());
        }
        args.join(" ")
    }
}

impl Extension for Nix {
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

        let args = self.build_args();

        Command::new("bash")
            .arg("-c")
            .arg(&format!("[ -f flake.nix ] || nix flake init {}", args))
            .current_dir(work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;

        let cmd = format!("nix develop {} -c {}", args, cmd);
        exec(&cmd, tx, out, last_cmd, work_dir)
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
            .arg("type systemctl > /dev/null 2> /dev/null")
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

    fn format_command(&self, cmd: &str) -> String {
        let args = self.build_args();
        format!(
            "[ -f flake.nix ] || nix flake init {} ; nix develop {} -c {}",
            args, args, cmd
        )
    }
}
