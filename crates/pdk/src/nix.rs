use extism_pdk::*;
use fluentci_types::{cache::Cache, nix as types};
use serde::{Deserialize, Serialize};

#[host_fn]
extern "ExtismHost" {
    fn set_runner(runner: String);
    fn with_exec(args: Json<Vec<String>>);
    fn with_workdir(path: String);
    fn with_cache(cache: Json<Cache>);
    fn stdout() -> String;
    fn stderr() -> String;
}

#[derive(Serialize, Deserialize)]
pub struct Nix {
    pub id: String,
}

impl From<types::Nix> for Nix {
    fn from(nix: types::Nix) -> Self {
        Nix { id: nix.id }
    }
}

impl Nix {
    pub fn with_exec(&self, args: Vec<String>) -> Result<Nix, Error> {
        unsafe { set_runner("nix".into()) }?;
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Nix {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Nix, Error> {
        unsafe { with_workdir(path) }?;
        Ok(Nix {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Nix, Error> {
        unsafe {
            with_cache(Json(Cache {
                id: cache_id,
                path,
                ..Default::default()
            }))
        }?;
        Ok(Nix {
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