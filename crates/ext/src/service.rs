use std::{process::ExitStatus, sync::mpsc::Sender};

use crate::{exec, pkgx::Pkgx, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Service {}

impl Service {
    pub fn get_random_port(&self) -> String {
        todo!()
    }
}

impl Extension for Service {
    fn exec(
        &mut self,
        yaml: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error> {
        self.setup()?;

        if yaml.is_empty() {
            return Ok(ExitStatus::default());
        }

        let port = self.get_random_port();
        let cmd = format!("pkgx process-compose up -t=false -p {}", port);
        exec(&cmd, tx, out, last_cmd, work_dir)
    }

    fn setup(&self) -> Result<(), Error> {
        Pkgx::default().setup()?;
        Ok(())
    }
}
