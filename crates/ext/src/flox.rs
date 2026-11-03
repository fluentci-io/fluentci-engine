use std::{
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, Extension};
use anyhow::{anyhow, Context, Error};
use fluentci_types::Output;

#[derive(Default)]
pub struct Flox {}

fn verify(command: &mut Command) -> Result<(), Error> {
    let output = command.arg("--version").output().context(
        "Flox is unavailable; install the native package from https://flox.dev/download/ and put flox on PATH",
    )?;
    if !output.status.success() {
        return Err(anyhow!("Flox version check failed ({}); repair the native Flox installation before executing pipelines", output.status));
    }
    Ok(())
}

fn initialize(command: &mut Command, work_dir: &str) -> Result<(), Error> {
    if std::path::Path::new(work_dir).join(".flox").is_dir() {
        return Ok(());
    }
    let status = command
        .arg("init")
        .current_dir(work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("could not initialize Flox environment")?;
    if !status.success() {
        return Err(anyhow!(
            "Flox initialization failed ({status}); command was not executed"
        ));
    }
    Ok(())
}

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
        initialize(&mut Command::new("flox"), work_dir)?;
        exec(
            &format!("flox activate -- {cmd}"),
            tx,
            out,
            last_cmd,
            work_dir,
        )
    }

    fn setup(&self) -> Result<(), Error> {
        // Current native Flox packages manage their own Nix installation. Do not
        // bootstrap a second Nix or modify system-wide trust/cache settings here.
        verify(&mut Command::new("flox"))
    }

    fn format_command(&self, cmd: &str) -> String {
        format!("([ -d .flox ] || flox init) && flox activate -- {cmd}")
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn flox_setup_and_initialization_fail_before_activation() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing");
        assert!(verify(&mut Command::new(missing))
            .unwrap_err()
            .to_string()
            .contains("native package"));
        let mut failed = Command::new("/bin/sh");
        failed.args(["-c", "exit 7", "fixture"]);
        assert!(verify(&mut failed)
            .unwrap_err()
            .to_string()
            .contains("version check failed"));
        let mut installed = Command::new("/bin/sh");
        installed.args(["-c", "test \"$1\" = --version", "fixture"]);
        verify(&mut installed).unwrap();
        let mut failed = Command::new("/bin/sh");
        failed.args(["-c", "exit 9", "fixture"]);
        assert!(initialize(&mut failed, root.path().to_str().unwrap())
            .unwrap_err()
            .to_string()
            .contains("command was not executed"));
        let mut success = Command::new("/bin/sh");
        success.args(["-c", "test \"$1\" = init && /bin/mkdir .flox", "fixture"]);
        initialize(&mut success, root.path().to_str().unwrap()).unwrap();
        initialize(
            &mut Command::new("/not-invoked"),
            root.path().to_str().unwrap(),
        )
        .unwrap();
        let activation = tempfile::tempdir().unwrap();
        let script = format!("flox() {{ if [ \"$1\" = init ]; then return 9; fi; printf activated > activated; }}; {}", Flox::default().format_command("true"));
        let status = Command::new("/bin/sh")
            .args(["-c", &script])
            .current_dir(activation.path())
            .status()
            .unwrap();
        assert!(!status.success());
        assert!(!activation.path().join("activated").exists());
    }
}
