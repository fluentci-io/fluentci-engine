use crate::Extension;
use anyhow::Error;

#[derive(Default)]
pub struct Flox {}

impl Extension for Flox {
    fn exec(&self) -> Result<(), Error> {
        Ok(())
    }

    fn setup(&self) -> Result<(), Error> {
        Ok(())
    }
}
