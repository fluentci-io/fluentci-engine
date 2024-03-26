use extism_pdk::*;
use fluentci_types::pixi as types;
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
pub struct Pixi {
    pub id: String,
}

impl From<types::Pixi> for Pixi {
    fn from(pixi: types::Pixi) -> Self {
        Pixi { id: pixi.id }
    }
}

impl Pixi {
    pub fn with_exec(&self, args: Vec<String>) -> Result<Pixi, Error> {
        unsafe { set_runner("pixi".into()) }?;
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Pixi {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Pixi, Error> {
        unsafe { with_workdir(path) }?;
        Ok(Pixi {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Pixi, Error> {
        unsafe { with_cache(path, cache_id) }?;
        Ok(Pixi {
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
