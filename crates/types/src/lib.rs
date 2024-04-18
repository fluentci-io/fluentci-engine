use serde::{Deserialize, Serialize};

pub mod cache;
pub mod devbox;
pub mod devenv;
pub mod directory;
pub mod envhub;
pub mod file;
pub mod flox;
pub mod git;
pub mod mise;
pub mod nix;
pub mod pipeline;
pub mod pixi;
pub mod pkgx;
pub mod proto;
pub mod service;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Output {
    Stdout,
    Stderr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Module {
    pub url: String,
    pub function: String,
    pub args: String,
}
