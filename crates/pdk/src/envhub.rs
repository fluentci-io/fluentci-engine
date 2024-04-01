use extism_pdk::*;
use fluentci_types::{cache::Cache, envhub as types};
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
pub struct Envhub {
    pub id: String,
}

impl From<types::Envhub> for Envhub {
    fn from(envhub: types::Envhub) -> Self {
        Envhub { id: envhub.id }
    }
}

impl Envhub {
    pub fn with_exec(&self, args: Vec<&str>) -> Result<Envhub, Error> {
        unsafe { set_runner("envhub".into()) }?;
        unsafe {
            with_exec(Json::from(
                args.into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ))
        }?;
        Ok(Envhub {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: &str) -> Result<Envhub, Error> {
        unsafe { with_workdir(path.into()) }?;
        Ok(Envhub {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: &str, cache_id: &str) -> Result<Envhub, Error> {
        unsafe {
            with_cache(Json(Cache {
                id: cache_id.into(),
                path: path.into(),
                ..Default::default()
            }))
        }?;
        Ok(Envhub {
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
