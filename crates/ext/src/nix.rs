use crate::Extension;
use anyhow::Error;

#[derive(Default)]
pub struct Nix {}

impl Extension for Nix {
    fn exec(&self) -> Result<(), Error> {
        Ok(())
    }

    fn setup(&self) -> Result<(), Error> {
        Ok(())
    }
}
