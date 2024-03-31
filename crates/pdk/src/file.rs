use extism_pdk::*;
use fluentci_types::file as types;
use serde::{Deserialize, Serialize};

use super::directory::Directory;

#[host_fn]
extern "ExtismHost" {
    fn zip(path: String) -> Json<File>;
    fn unzip(path: String) -> Json<Directory>;
    fn tar_czvf(path: String) -> Json<File>;
    fn tar_xzvf(path: String) -> Json<Directory>;
    fn md5(path: String) -> String;
    fn sha256(path: String) -> String;
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub path: String,
}

impl From<types::File> for File {
    fn from(file: types::File) -> Self {
        File {
            id: file.id,
            path: file.path,
        }
    }
}

impl File {
    pub fn zip(&self) -> Result<File, Error> {
        let file = unsafe { zip(self.path.clone()) }?;
        Ok(file.into_inner())
    }

    pub fn unzip(&self) -> Result<Directory, Error> {
        let directory = unsafe { unzip(self.path.clone()) }?;
        Ok(directory.into_inner())
    }

    pub fn tar_czvf(&self) -> Result<File, Error> {
        let file = unsafe { tar_czvf(self.path.clone()) }?;
        Ok(file.into_inner())
    }

    pub fn tar_xzvf(&self) -> Result<Directory, Error> {
        let directory = unsafe { tar_xzvf(self.path.clone()) }?;
        Ok(directory.into_inner())
    }

    pub fn md5(&self) -> Result<String, Error> {
        unsafe { md5(self.path.clone()) }
    }

    pub fn sha256(&self) -> Result<String, Error> {
        unsafe { sha256(self.path.clone()) }
    }
}
