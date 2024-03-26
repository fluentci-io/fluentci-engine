use extism_pdk::*;
use fluentci_types::devbox as types;
use serde::{Deserialize, Serialize};

#[host_fn]
extern "ExtismHost" {
    fn set_runner(runner: String);
    fn with_exec(args: Json<Vec<String>>);
    fn with_workdir(path: String);
    fn with_cache(path: String, cache_id: String);
    fn stdout() -> String;
    fn stderr() -> String;
}

#[derive(Serialize, Deserialize)]
pub struct Devbox {
    pub id: String,
}

impl From<types::Devbox> for Devbox {
    fn from(devbox: types::Devbox) -> Self {
        Devbox { id: devbox.id }
    }
}

impl Devbox {
    pub fn with_exec(&self, args: Vec<String>) -> Result<Devbox, Error> {
        unsafe { set_runner("devbox".into()) }?;
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Devbox {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Devbox, Error> {
        unsafe { with_workdir(path) }?;
        Ok(Devbox {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Devbox, Error> {
        unsafe { with_cache(path, cache_id) }?;
        Ok(Devbox {
            id: self.id.clone(),
        })
    }

    pub fn stdout(&self) -> Result<String, Error> {
        unsafe { stdout() }
    }

    pub fn stderr(&self) -> Result<String, Error> {
        unsafe { stderr() }
    }
}
