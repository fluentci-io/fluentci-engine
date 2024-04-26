use extism_pdk::*;
use fluentci_types::{cache::Cache, nix::NixArgs, pipeline as types, service::Service};
use serde::{Deserialize, Serialize};

use super::{
    devbox::Devbox, devenv::Devenv, envhub::Envhub, file::File, flox::Flox, git::Git, mise::Mise,
    nix::Nix, pixi::Pixi, pkgx::Pkgx,
};

#[host_fn]
extern "ExtismHost" {
    fn set_runner(runner: String);
    fn devbox() -> Json<Devbox>;
    fn devenv() -> Json<Devenv>;
    fn flox() -> Json<Flox>;
    fn nix(args: Json<NixArgs>) -> Json<Nix>;
    fn pkgx() -> Json<Pkgx>;
    fn pixi() -> Json<Pixi>;
    fn git(url: String) -> Json<Git>;
    fn http(url: String) -> Json<File>;
    fn mise() -> Json<Mise>;
    fn envhub() -> Json<Envhub>;
    fn with_exec(args: Json<Vec<String>>);
    fn with_workdir(path: String);
    fn with_cache(cache: Json<Cache>);
    fn with_file(file: Json<File>);
    fn stdout() -> String;
    fn stderr() -> String;
    fn as_service(name: String) -> Json<Service>;
    fn with_service(service_id: String);
}

#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
}

impl From<types::Pipeline> for Pipeline {
    fn from(pipeline: types::Pipeline) -> Self {
        Pipeline { id: pipeline.id }
    }
}

impl Pipeline {
    pub fn devbox(&self) -> Result<Devbox, Error> {
        unsafe { set_runner("devbox".into()) }?;
        let devbox = unsafe { devbox() }?;
        Ok(devbox.into_inner())
    }

    pub fn devenv(&self) -> Result<Devenv, Error> {
        unsafe { set_runner("devenv".into()) }?;
        let devenv = unsafe { devenv() }?;
        Ok(devenv.into_inner())
    }

    pub fn flox(&self) -> Result<Flox, Error> {
        unsafe { set_runner("flox".into()) }?;
        let flox = unsafe { flox() }?;
        Ok(flox.into_inner())
    }

    pub fn nix(&self, args: NixArgs) -> Result<Nix, Error> {
        unsafe { set_runner("nix".into()) }?;
        let nix = unsafe { nix(Json(args)) }?;
        Ok(nix.into_inner())
    }

    pub fn pkgx(&self) -> Result<Pkgx, Error> {
        unsafe { set_runner("pkgx".into()) }?;
        let pkgx = unsafe { pkgx() }?;
        Ok(pkgx.into_inner())
    }

    pub fn pixi(&self) -> Result<Pixi, Error> {
        unsafe { set_runner("pixi".into()) }?;
        let pixi = unsafe { pixi() }?;
        Ok(pixi.into_inner())
    }

    pub fn git(&self, url: &str) -> Result<Git, Error> {
        unsafe { set_runner("git".into()) }?;
        let git = unsafe { git(url.into()) }?;
        Ok(git.into_inner())
    }

    pub fn http(&self, url: &str) -> Result<File, Error> {
        unsafe { set_runner("http".into()) }?;
        let file = unsafe { http(url.into()) }?;
        Ok(file.into_inner())
    }

    pub fn mise(&self) -> Result<Mise, Error> {
        unsafe { set_runner("mise".into()) }?;
        let mise = unsafe { mise() }?;
        Ok(mise.into_inner())
    }

    pub fn envhub(&self) -> Result<Envhub, Error> {
        unsafe { set_runner("envhub".into()) }?;
        let envhub = unsafe { envhub() }?;
        Ok(envhub.into_inner())
    }

    pub fn with_exec(&self, args: Vec<&str>) -> Result<Pipeline, Error> {
        unsafe {
            with_exec(Json::from(
                args.into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            ))
        }?;
        Ok(Pipeline {
            id: self.id.clone(),
        })
    }

    pub fn with_workdir(&self, path: &str) -> Result<Pipeline, Error> {
        unsafe { with_workdir(path.into()) }?;
        Ok(Pipeline {
            id: self.id.clone(),
        })
    }

    pub fn with_cache(&self, path: &str, cache_id: &str) -> Result<Pipeline, Error> {
        unsafe {
            with_cache(Json(Cache {
                id: cache_id.into(),
                path: path.into(),
                ..Default::default()
            }))
        }?;
        Ok(Pipeline {
            id: self.id.clone(),
        })
    }

    pub fn with_file(&self, path: &str, file_id: &str) -> Result<Pipeline, Error> {
        unsafe {
            with_file(Json(File {
                id: file_id.into(),
                path: path.into(),
            }))
        }?;
        Ok(Pipeline {
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

    pub fn with_service(&self, service_id: &str) -> Result<Pipeline, Error> {
        unsafe { with_service(service_id.into())? }
        Ok(Pipeline {
            id: self.id.clone(),
        })
    }
}
