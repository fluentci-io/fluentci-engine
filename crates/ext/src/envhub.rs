use std::{
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::Sender,
};

use crate::{bootstrap, exec, nix::Nix, Extension};
use anyhow::{anyhow, Context, Error};
use fluentci_types::Output;

#[derive(Default)]
pub struct Envhub {}

impl Envhub {
    pub fn r#use(&self, environment: &str, work_dir: &str) -> Result<ExitStatus, Error> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(format!("envhub use {}", environment))
            .current_dir(work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        child.wait().map_err(Error::from)
    }
}

impl Extension for Envhub {
    fn exec(
        &mut self,
        cmd: &str,
        tx: Sender<String>,
        out: Output,
        last_cmd: bool,
        work_dir: &str,
    ) -> Result<ExitStatus, Error> {
        self.setup()?;

        if cmd.is_empty() {
            return Ok(ExitStatus::default());
        }

        exec(cmd, tx, out, last_cmd, work_dir)
    }

    fn setup(&self) -> Result<(), Error> {
        Nix::default().setup()?;
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Envhub installation requires a home directory"))?;
        let bin = home.join(".local/bin");
        let mut paths: Vec<_> =
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
        if !paths.contains(&bin) {
            paths.push(bin.clone());
            std::env::set_var("PATH", std::env::join_paths(paths)?);
        }
        let available = Command::new("/bin/sh")
            .args(["-c", "command -v envhub >/dev/null 2>&1"])
            .status()?;
        if available.success() {
            return Ok(());
        }
        let (target, checksum) = release_for(std::env::consts::OS, std::env::consts::ARCH)?;
        let temporary = tempfile::tempdir()?;
        let archive = temporary.path().join("envhub.tar.gz");
        let url = format!("https://github.com/tsirysndr/envhub/releases/download/v0.3.0/envhub_v0.3.0_{target}.tar.gz");
        bootstrap::download(&url, &archive)?;
        verify_archive(&archive, checksum)?;
        install_archive(&archive, &bin.join("envhub"))?;
        let status = Command::new(bin.join("envhub")).arg("--version").status()?;
        if !status.success() {
            return Err(anyhow!(
                "Envhub release binary verification failed ({status})"
            ));
        }
        Ok(())
    }
}

fn release_for(os: &str, arch: &str) -> Result<(&'static str, &'static str), Error> {
    match (os, arch) {
        ("linux", "x86_64") => Ok((
            "x86_64-unknown-linux-gnu",
            "b5277e7c4c19ab33ae3e1f64e0310b0ac842e81ac4a73a5488239ec0d4620770",
        )),
        ("linux", "aarch64") => Ok((
            "aarch64-unknown-linux-gnu",
            "907530ce2c663cbf7b61f9fb0abed7dd60e24b57d0eacbcd54c65c19863fdbde",
        )),
        ("macos", "x86_64") => Ok((
            "x86_64-apple-darwin",
            "a043429f801bbff0a8fcd2d0f13254d8c686306f78637e90b061a6a4261112a1",
        )),
        ("macos", "aarch64") => Ok((
            "aarch64-apple-darwin",
            "75284bb43865a7506bca65fe79ee802e9943fde2af1e2da87e9285541dfca7df",
        )),
        _ => Err(anyhow!("no pinned Envhub binary for {os}/{arch}")),
    }
}

fn verify_archive(path: &std::path::Path, expected: &str) -> Result<(), Error> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if hex_digest(digest.finalize().as_slice()) != expected {
        return Err(anyhow!("Envhub release archive checksum mismatch"));
    }
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn install_archive(archive: &std::path::Path, destination: &std::path::Path) -> Result<(), Error> {
    let extracted = tempfile::tempdir()?;
    let status = Command::new("/usr/bin/tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(extracted.path())
        .arg("envhub")
        .status()?;
    if !status.success() {
        return Err(anyhow!("Envhub archive extraction failed ({status})"));
    }
    let executable = extracted.path().join("envhub");
    if !std::fs::symlink_metadata(&executable)?
        .file_type()
        .is_file()
    {
        return Err(anyhow!(
            "Envhub archive does not contain a regular executable"
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| anyhow!("invalid Envhub destination"))?;
    std::fs::create_dir_all(parent)?;
    let staging = tempfile::NamedTempFile::new_in(parent)?;
    std::fs::copy(executable, staging.path())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(staging.path(), std::fs::Permissions::from_mode(0o755))?;
    }
    staging
        .persist(destination)
        .context("could not publish Envhub binary")?;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn github_release_platforms_and_checksum_validation() {
        for os in ["linux", "macos"] {
            for arch in ["x86_64", "aarch64"] {
                let (target, checksum) = release_for(os, arch).unwrap();
                assert!(target.starts_with(arch));
                assert_eq!(checksum.len(), 64);
            }
        }
        assert!(release_for("freebsd", "x86_64").is_err());
        assert!(release_for("linux", "arm").is_err());
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("download");
        std::fs::write(&file, "bad download").unwrap();
        assert!(verify_archive(&file, release_for("linux", "x86_64").unwrap().1).is_err());
    }
    #[test]
    fn installs_only_archive_binary_into_requested_directory() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("envhub"), "#!/bin/sh\nexit 0\n").unwrap();
        let archive = root.path().join("fixture.tar.gz");
        assert!(Command::new("/usr/bin/tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(root.path())
            .arg("envhub")
            .status()
            .unwrap()
            .success());
        let destination = root.path().join("bin/envhub");
        install_archive(&archive, &destination).unwrap();
        assert!(Command::new(&destination)
            .arg("--version")
            .status()
            .unwrap()
            .success());
    }
}
