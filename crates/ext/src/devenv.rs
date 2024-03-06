use crate::Extension;
use anyhow::Error;

#[derive(Default)]
pub struct Devenv {}

impl Extension for Devenv {
    fn exec(&self) -> Result<(), Error> {
        Ok(())
    }

    fn setup(&self) -> Result<(), Error> {
        Ok(())
    }
}
