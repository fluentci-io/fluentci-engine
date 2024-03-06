use crate::Extension;
use anyhow::Error;

#[derive(Default)]
pub struct Pkgx {}

impl Extension for Pkgx {
    fn exec(&self) -> Result<(), Error> {
        Ok(())
    }

    fn setup(&self) -> Result<(), Error> {
        Ok(())
    }
}
