use std::{
    fs::{self, metadata},
    path::Path,
    process::ExitStatus,
    sync::mpsc::Sender,
};

use crate::{exec, pkgx::Pkgx, Extension};
use anyhow::Error;
use fluentci_types::Output;
use owo_colors::OwoColorize;

#[derive(Default)]
pub struct Cache {
    key: String,
    path: String,
}

impl Extension for Cache {
    fn exec(
        &mut self,
        key: &str,
        tx: Sender<String>,
        _out: Output,
        _last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error> {
        let key_path = key.split(":").collect::<Vec<&str>>();

        if key_path.len() != 2 {
            return Err(Error::msg("Invalid cache key"));
        }

        self.key = key_path[0].to_string();
        self.path = match key_path[1].starts_with("/") {
            true => key_path[1].to_string(),
            false => format!("{}/{}", work_dir, key_path[1]),
        };

        println!("Cache key: {}", key);

        let cache_file = format!(
            "{}/.fluentci/cache/{}.tar.gz",
            dirs::home_dir().unwrap().to_str().unwrap(),
            self.key
        );

        if Path::new(&cache_file).exists() {
            let parent = Path::new(&self.path).parent().unwrap();
            let work_dir = String::from(parent.to_str().unwrap());

            fs::create_dir_all(&work_dir)?;

            let cmd = format!("tar xzvf {}", cache_file);
            exec(&cmd, tx, Output::Stdout, true, &work_dir)?;
            let label = format!("[{}]", "withCache");
            println!("{} Cached restored", label.cyan());
            return Ok(ExitStatus::default());
        }

        tx.send("Cached restored".to_string())?;
        Ok(ExitStatus::default())
    }

    fn setup(&self) -> Result<(), Error> {
        Pkgx::default().install(vec!["tar"])?;
        Ok(())
    }

    fn post_setup(&self, tx: Sender<String>) -> Result<ExitStatus, Error> {
        println!("Cache key: {}", self.key);
        println!("Cache path: {}", self.path);

        fs::create_dir_all(format!(
            "{}/.fluentci/cache",
            dirs::home_dir().unwrap().to_str().unwrap()
        ))?;

        let output_file = format!(
            "{}/.fluentci/cache/{}.tar.gz",
            dirs::home_dir().unwrap().to_str().unwrap(),
            self.key
        );

        if let Some(parent) = Path::new(&self.path).parent() {
            let (is_dir, is_file) =
                metadata(&self.path).map(|metadata| (metadata.is_dir(), metadata.is_file()))?;

            if let Some(file_or_dir) = Path::new(&self.path).file_name() {
                let cmd = match (is_dir, is_file) {
                    (true, false) => {
                        format!("tar czvf {} {}", output_file, file_or_dir.to_str().unwrap())
                    }
                    (false, true) => {
                        format!("tar czvf {} {}", output_file, file_or_dir.to_str().unwrap())
                    }
                    _ => return Err(Error::msg("Invalid file or directory")),
                };

                let work_dir = String::from(parent.to_str().unwrap());

                return exec(&cmd, tx, Output::Stdout, true, &work_dir);
            }
            return Err(Error::msg("Failed to get file or directory name"));
        }

        Err(Error::msg("Failed to get parent directory"))
    }
}
