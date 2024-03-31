use extism_pdk::*;
use fluentci_types::flox as types;
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
pub struct Flox {
    pub id: String,
}

impl From<types::Flox> for Flox {
    fn from(flox: types::Flox) -> Self {
        Flox { id: flox.id }
    }
}

impl Flox {
    pub fn with_exec(&self, args: Vec<String>) -> Result<Flox, Error> {
        unsafe { set_runner("flox".into()) }?;
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Flox {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Flox, Error> {
        unsafe { with_workdir(path) }?;
        Ok(Flox {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Flox, Error> {
        unsafe { with_cache(path, cache_id) }?;
        Ok(Flox {
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
