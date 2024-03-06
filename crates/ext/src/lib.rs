use anyhow::Error;

pub mod devbox;
pub mod devenv;
pub mod flox;
pub mod nix;
pub mod pkgx;

pub trait Extension {
    fn exec(&self) -> Result<(), Error>;
    fn setup(&self) -> Result<(), Error>;
}
