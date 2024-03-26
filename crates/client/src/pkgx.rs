use extism_pdk::*;
use fluentci_types::pkgx as types;
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
pub struct Pkgx {
    pub id: String,
}

impl From<types::Pkgx> for Pkgx {
    fn from(pkgx: types::Pkgx) -> Self {
        Pkgx { id: pkgx.id }
    }
}

impl Pkgx {
    pub fn with_exec(&self, args: Vec<String>) -> Result<Pkgx, Error> {
        unsafe { set_runner("pkgx".into()) }?;
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Pkgx {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Pkgx, Error> {
        unsafe { with_workdir(path) }?;
        Ok(Pkgx {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Pkgx, Error> {
        unsafe { with_cache(path, cache_id) }?;
        Ok(Pkgx {
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
