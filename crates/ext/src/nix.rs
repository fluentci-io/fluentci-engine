use std::{process::ExitStatus, sync::mpsc::Sender};

use crate::Extension;
use anyhow::Error;
use fluentci_types::Output;

#[derive(Default)]
pub struct Nix {}

impl Extension for Nix {
    fn exec(
        &self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
    ) -> Result<ExitStatus, Error> {
        todo!()
    }

    fn setup(&self) -> Result<(), Error> {
        Ok(())
    }
}
