use std::{
    env,
    fs::{self, metadata},
    path::Path,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Sender},
    thread,
};

use crate::{exec, pkgx::Pkgx, Extension};
use anyhow::Error;
use async_trait::async_trait;
use fluentci_types::Output;
use owo_colors::OwoColorize;

pub mod gcs;
pub mod s3;

pub enum CacheBackendType {
    CDN,
    S3,
    GCS,
}

#[async_trait]
pub trait CacheBackend {
    async fn restore(&self, path: &str) -> Result<(), Error>;
    async fn save(&self, path: &str) -> Result<(), Error>;
}

#[derive(Default)]
pub struct Cache {
    key: String,
    path: String,
}

pub fn detect_cache_backend() -> Option<CacheBackendType> {
    if let Ok(_) = std::env::var("FLUENTCI_CACHE_CDN_ENDPOINT") {
        return Some(CacheBackendType::CDN);
    }

    if let Ok(_) = std::env::var("FLUENTCI_CACHE_S3_ENDPOINT") {
        return Some(CacheBackendType::S3);
    }

    if let Ok(_) = std::env::var("FLUENTCI_CACHE_GCS_BUCKET") {
        return Some(CacheBackendType::GCS);
    }

    None
}

pub async fn download(key: &str) -> Result<(), Error> {
    let default_backend: Option<CacheBackendType> = detect_cache_backend();

    if default_backend.is_none() {
        println!("-> No cache backend found, skipping download");
        return Ok(());
    }

    let cache_file = format!(
        "{}/.fluentci/cache/{}.tar.gz",
        dirs::home_dir().unwrap().to_str().unwrap(),
        key
    );

    if Path::new(&cache_file).exists() {
        println!("-> Cache already exists, skipping download");
        return Ok(());
    }

    println!(
        " {} cache for key: {} ...",
        "Downloading".bright_green(),
        key.bright_yellow()
    );

    if let Some(CacheBackendType::CDN) = default_backend {
        fs::create_dir_all(format!(
            "{}/.fluentci/cache",
            dirs::home_dir().unwrap().to_str().unwrap()
        ))?;

        let prefix = format!("{}/{}", env::consts::OS, env::consts::ARCH);
        let url = std::env::var("FLUENTCI_CACHE_CDN_ENDPOINT")?;
        let filename = cache_file.split("/").last().unwrap();
        let url = format!("{}/{}/{}", url, prefix, filename);
        let cmd = format!("wget {} -O {}", url, cache_file);
        println!("-> {}", cmd.bright_cyan());

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        child.wait()?;

        return Ok(());
    }

    let backend: Box<dyn CacheBackend> = match default_backend {
        Some(CacheBackendType::S3) => Box::new(s3::new().await?),
        Some(CacheBackendType::GCS) => Box::new(gcs::new().await?),
        _ => Box::new(s3::new().await?),
    };
    backend.restore(&cache_file).await?;
    Ok(())
}

pub async fn upload(key: &str) -> Result<(), Error> {
    let mut default_backend: Option<&str> = None;

    if let Ok(_) = std::env::var("FLUENTCI_CACHE_S3_ENDPOINT") {
        default_backend = Some("S3");
    }

    if let Ok(_) = std::env::var("FLUENTCI_CACHE_GCS_ENDPOINT") {
        default_backend = Some("GCS");
    }

    if default_backend.is_none() {
        println!("-> No cache backend found, skipping upload");
        return Ok(());
    }

    println!(
        " {} cache for key: {} ...",
        "Uploading".bright_green(),
        key.bright_yellow()
    );
    let cache_file = format!(
        "{}/.fluentci/cache/{}.tar.gz",
        dirs::home_dir().unwrap().to_str().unwrap(),
        key
    );
    let backend: Box<dyn CacheBackend> = match default_backend {
        Some("S3") => Box::new(s3::new().await?),
        Some("GCS") => Box::new(gcs::new().await?),
        _ => Box::new(s3::new().await?),
    };
    backend.save(&cache_file).await?;
    Ok(())
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

        if !Path::new(&cache_file).exists() {
            let key = self.key.clone();
            let (dtx, drx) = mpsc::channel();
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(download(&key)).unwrap();
                dtx.send(0).unwrap();
            });
            drx.recv().unwrap();
        }

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
        Pkgx::default().install(vec!["tar", "wget"])?;
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

                exec(&cmd, tx, Output::Stdout, true, &work_dir)?;

                let key = self.key.clone();
                let (utx, urx) = mpsc::channel();
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(upload(&key)).unwrap();
                    utx.send(0).unwrap();
                });
                urx.recv().unwrap();

                return Ok(ExitStatus::default());
            }
            return Err(Error::msg("Failed to get file or directory name"));
        }

        Err(Error::msg("Failed to get parent directory"))
    }
}
