use std::{fs, process::ExitStatus, sync::mpsc::Sender};

use crate::{exec, pkgx::Pkgx, Extension};
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Service {}

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

        fs::create_dir_all(work_dir)?;

        let path = format!("{}/process-compose.yaml", work_dir);
        fs::write(&path, yaml)?;

        exec(
            "pkgx process-compose up -t=false -u process-compose.sock",
            tx,
            out,
            last_cmd,
            work_dir,
        )
    }

    fn setup(&self) -> Result<(), Error> {
        Pkgx::default().setup()?;
        Ok(())
    }

    fn post_setup(&self, tx: Sender<String>) -> Result<ExitStatus, Error> {
        exec(
            "pkgx process-compose down -u process-compose.sock",
            tx,
            Output::Stdout,
            true,
            ".fluentci",
        )
    }
}
