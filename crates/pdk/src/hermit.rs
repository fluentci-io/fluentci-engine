use extism_pdk::*;
use fluentci_types::{cache::Cache, file::File, hermit as types, service::Service};
use serde::{Deserialize, Serialize};

#[host_fn]
extern "ExtismHost" {
    fn set_runner(runner: String);
    fn with_exec(args: Json<Vec<String>>);
    fn with_workdir(path: String);
    fn with_cache(cache: Json<Cache>);
    fn with_file(file: Json<File>);
    fn with_packages(packages: Json<Vec<String>>);
    fn stdout() -> String;
    fn stderr() -> String;
    fn as_service(name: String) -> Json<Service>;
    fn with_service(service_id: String);
    fn set_envs(envs: Json<Vec<(String, String)>>);
    fn wait_on(args: Json<Vec<u32>>);
    fn with_secret_variable(params: Json<Vec<String>>);
}

#[derive(Serialize, Deserialize)]
pub struct Hermit {
    pub id: String,
}

impl From<types::Hermit> for Hermit {
    fn from(hermit: types::Hermit) -> Self {
        Hermit { id: hermit.id }
    }
}

impl Hermit {
    pub fn with_exec(&self, args: Vec<&str>) -> Result<Hermit, Error> {
        unsafe { set_runner("hermit".into()) }?;
        unsafe {
            with_exec(Json::from(
                args.into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ))
        }?;
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn with_file(&self, path: &str, file_id: &str) -> Result<Hermit, Error> {
        unsafe {
            with_file(Json(File {
                id: file_id.into(),
                path: path.into(),
            }))
        }?;
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: &str) -> Result<Hermit, Error> {
        unsafe { with_workdir(path.into()) }?;
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: &str, cache_id: &str) -> Result<Hermit, Error> {
        unsafe {
            with_cache(Json(Cache {
                id: cache_id.into(),
                path: path.into(),
                ..Default::default()
            }))
        }?;
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn with_packages(&self, packages: Vec<&str>) -> Result<Hermit, Error> {
        unsafe {
            with_packages(Json::from(
                packages
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ))
        }?;
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn stdout(&self) -> Result<String, Error> {
        unsafe { stdout() }
    }

    pub fn stderr(&self) -> Result<String, Error> {
        unsafe { stderr() }
    }

    pub fn as_service(&self, name: &str) -> Result<String, Error> {
        let service = unsafe { as_service(name.into())? };
        Ok(service.into_inner().id)
    }

    pub fn with_service(&self, service_id: &str) -> Result<Hermit, Error> {
        unsafe { with_service(service_id.into())? }
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn with_env_variable(&self, name: &str, value: &str) -> Result<Hermit, Error> {
        unsafe {
            set_envs(Json(vec![(name.into(), value.into())]))?;
        }
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn wait_on(&self, port: u32, timeout: Option<u32>) -> Result<Hermit, Error> {
        unsafe {
            wait_on(Json(vec![port, timeout.unwrap_or(60)]))?;
        }
        Ok(Hermit {
            id: self.id.clone(),
        })
    }

    pub fn with_secret_variable(&self, name: &str, secret_id: &str) -> Result<Hermit, Error> {
        unsafe {
            with_secret_variable(Json(vec![name.into(), secret_id.into()]))?;
        }
        Ok(Hermit {
            id: self.id.clone(),
        })
    }
}
