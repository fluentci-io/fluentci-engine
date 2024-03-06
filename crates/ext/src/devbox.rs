use crate::Extension;
use anyhow::Error;

#[derive(Default)]
pub struct Devbox {}

impl Extension for Devbox {
    fn exec(&self) -> Result<(), Error> {
        Ok(())
    }

    fn setup(&self) -> Result<(), Error> {
        Ok(())
    }
}
