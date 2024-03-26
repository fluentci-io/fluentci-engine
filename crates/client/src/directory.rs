use extism_pdk::*;
use fluentci_types::directory as types;
use serde::{Deserialize, Serialize};

use super::{
    devbox::Devbox, devenv::Devenv, envhub::Envhub, file::File, flox::Flox, mise::Mise, nix::Nix,
    pixi::Pixi, pkgx::Pkgx,
};

#[host_fn]
extern "ExtismHost" {
    fn directory(path: String) -> Json<Directory>;
    fn entries(path: String) -> Json<Vec<String>>;
    fn devbox() -> Json<Devbox>;
    fn devenv() -> Json<Devenv>;
    fn flox() -> Json<Flox>;
    fn nix() -> Json<Nix>;
    fn pkgx() -> Json<Pkgx>;
    fn pixi() -> Json<Pixi>;
    fn mise() -> Json<Mise>;
    fn envhub() -> Json<Envhub>;
    fn tar_czvf(path: String) -> Json<File>;
    fn zip(path: String) -> Json<File>;
    fn with_exec(args: Json<Vec<String>>);
    fn with_workdir(path: String);
    fn with_cache(path: String, cache_id: String);
    fn stdout() -> String;
    fn stderr() -> String;
}

#[derive(Serialize, Deserialize)]
pub struct Directory {
    pub id: String,
    pub path: String,
}

impl From<types::Directory> for Directory {
    fn from(directory: types::Directory) -> Self {
        Directory {
            id: directory.id,
            path: directory.path,
        }
    }
}

impl Directory {
    pub fn directory(&self, path: String) -> Result<Directory, Error> {
        let directory = unsafe { directory(path.clone()) }?;
        Ok(directory.into_inner())
    }

    pub fn entries(&self) -> Result<Vec<String>, Error> {
        let entries = unsafe { entries(self.path.clone()) }?;
        Ok(entries.into_inner())
    }

    pub fn devbox(&self) -> Result<Devbox, Error> {
        let devbox = unsafe { devbox() }?;
        Ok(devbox.into_inner())
    }

    pub fn devenv(&self) -> Result<Devenv, Error> {
        let devenv = unsafe { devenv() }?;
        Ok(devenv.into_inner())
    }

    pub fn flox(&self) -> Result<Flox, Error> {
        let flox = unsafe { flox() }?;
        Ok(flox.into_inner())
    }

    pub fn nix(&self) -> Result<Nix, Error> {
        let nix = unsafe { nix() }?;
        Ok(nix.into_inner())
    }

    pub fn pkgx(&self) -> Result<Pkgx, Error> {
        let pkgx = unsafe { pkgx() }?;
        Ok(pkgx.into_inner())
    }

    pub fn pixi(&self) -> Result<Pixi, Error> {
        let pixi = unsafe { pixi() }?;
        Ok(pixi.into_inner())
    }

    pub fn mise(&self) -> Result<Mise, Error> {
        let mise = unsafe { mise() }?;
        Ok(mise.into_inner())
    }

    pub fn envhub(&self) -> Result<Envhub, Error> {
        let envhub = unsafe { envhub() }?;
        Ok(envhub.into_inner())
    }

    pub fn tar_czvf(&self) -> Result<File, Error> {
        let file = unsafe { tar_czvf(self.path.clone()) }?;
        Ok(file.into_inner())
    }

    pub fn zip(&self) -> Result<File, Error> {
        let file = unsafe { zip(self.path.clone()) }?;
        Ok(file.into_inner())
    }

    pub fn with_exec(&self, args: Vec<String>) -> Result<Directory, Error> {
        unsafe { with_exec(Json::from(args)) }?;
        Ok(Directory {
            id: self.id.clone(),
            path: self.path.clone(),
        })
    }

    pub fn with_workdir(&self, path: String) -> Result<Directory, Error> {
        unsafe { with_workdir(path.clone()) }?;
        Ok(Directory {
            id: self.id.clone(),
            path,
        })
    }

    pub fn with_cache(&self, path: String, cache_id: String) -> Result<Directory, Error> {
        unsafe { with_cache(path.clone(), cache_id) }?;
        Ok(Directory {
            id: self.id.clone(),
            path: path,
        })
    }

    pub fn stdout(&self) -> Result<String, Error> {
        unsafe { stdout() }
    }

    pub fn stderr(&self) -> Result<String, Error> {
        unsafe { stderr() }
    }
}
