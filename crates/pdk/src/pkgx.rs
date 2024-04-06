use extism_pdk::*;
use fluentci_types::{cache::Cache, file::File, pkgx as types};
use serde::{Deserialize, Serialize};

#[host_fn]
extern "ExtismHost" {
    fn set_runner(runner: String);
    fn with_exec(args: Json<Vec<String>>);
    fn with_workdir(path: String);
    fn with_cache(cache: Json<Cache>);
    fn with_file(file: Json<File>);
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
    pub fn with_exec(&self, args: Vec<&str>) -> Result<Pkgx, Error> {
        unsafe { set_runner("pkgx".into()) }?;
        unsafe {
            with_exec(Json::from(
                args.into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ))
        }?;
        Ok(Pkgx {
            id: self.id.clone(),
        })
    }

    pub fn with_file(&self, path: &str, file_id: &str) -> Result<Pkgx, Error> {
        unsafe {
            with_file(Json(File {
                id: file_id.into(),
                path: path.into(),
            }))
        }?;
        Ok(Pkgx {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: &str) -> Result<Pkgx, Error> {
        unsafe { with_workdir(path.into()) }?;
        Ok(Pkgx {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: &str, cache_id: &str) -> Result<Pkgx, Error> {
        unsafe {
            with_cache(Json(Cache {
                id: cache_id.into(),
                path: path.into(),
                ..Default::default()
            }))
        }?;
        Ok(Pkgx {
            id: self.id.clone(),
        })
    }

    pub fn with_packages(&self, packages: Vec<&str>) -> Result<Pkgx, Error> {
        unsafe {
            with_exec(Json::from(
                packages
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ))
        }?;
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
