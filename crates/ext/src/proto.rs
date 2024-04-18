use std::{
    env,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{exec, pkgx::Pkgx, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Proto {}

impl Extension for Proto {
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

        exec(&cmd, tx, out, last_cmd, work_dir)
    }

    fn setup(&self) -> Result<(), Error> {
        let pkgx = Pkgx::default();
        pkgx.setup()?;

        let proto_home = format!("{}/.proto", env::var("HOME")?);
        env::set_var("PROTO_HOME", &proto_home);

        let path = format!(
            "{}:{}:{}",
            format!("{}/shims", proto_home),
            format!("{}/bin", proto_home),
            env::var("PATH")?,
        );

        env::set_var("PATH", &path);

        pkgx.install(vec!["unzip", "zip", "git", "xz"])?;

        let mut child = Command::new("sh")
            .arg("-c")
            .arg("type proto > /dev/null || curl -fsSL https://moonrepo.dev/install/proto.sh | bash -s -- --yes")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait()?;

        Ok(())
    }
}
