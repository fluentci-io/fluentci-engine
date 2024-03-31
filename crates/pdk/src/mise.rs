use extism_pdk::*;
use fluentci_types::{cache::Cache, mise as types};
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
pub struct Mise {
    pub id: String,
}

impl From<types::Mise> for Mise {
    fn from(mise: types::Mise) -> Self {
        Mise { id: mise.id }
    }
}

impl Mise {
    pub fn with_exec(&self, args: Vec<String>) -> Result<Mise, Error> {
        unsafe { set_runner("mise".into()) }?;
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Mise {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Mise, Error> {
        unsafe { with_workdir(path) }?;
        Ok(Mise {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Mise, Error> {
        unsafe {
            with_cache(Json(Cache {
                id: cache_id,
                path,
                ..Default::default()
            }))
        }?;
        Ok(Mise {
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
