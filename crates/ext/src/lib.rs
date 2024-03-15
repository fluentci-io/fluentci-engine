use std::{process::ExitStatus, sync::mpsc::Sender};

use anyhow::Error;
use fluentci_types::Output;

pub mod devbox;
pub mod devenv;
pub mod envhub;
pub mod flox;
pub mod nix;
pub mod pkgx;
pub mod runner;

pub trait Extension {
    fn exec(
        &self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error>;
    fn setup(&self) -> Result<(), Error>;
}
