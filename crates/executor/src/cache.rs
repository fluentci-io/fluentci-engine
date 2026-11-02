//! Opt-in local action cache. Cacheability is a contract: all consumed inputs and
//! produced outputs must be declared. This is not a filesystem/network sandbox.
use crate::Task;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CachePolicy {
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
    /// Caller-provided identity for the complete toolchain/runtime, not just the launcher.
    pub toolchain: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CacheResult {
    pub outcome: String,
    pub key: Option<String>,
    pub reason: String,
    /// Component digests explain key changes without exposing environment values.
    pub components: BTreeMap<String, String>,
}
impl CacheResult {
    pub fn disabled() -> Self {
        Self {
            outcome: "disabled".into(),
            reason: "task has no cache policy".into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: PathBuf,
    directory: bool,
    mode: u32,
    digest: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    key: String,
    outputs: Vec<Entry>,
    stdout: String,
    stderr: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ActionRecord {
    checksum: String,
    manifest: Manifest,
}

pub struct Prepared {
    pub key: String,
    pub(crate) executable: PathBuf,
    pub components: BTreeMap<String, String>,
    root: PathBuf,
    // Process-scoped advisory lock; released on drop, including crashes.
    _lock: File,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn file_digest(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}
fn encoded_digest<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(digest(
        &serde_json::to_vec(value).map_err(|e| e.to_string())?,
    ))
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|p| matches!(p, Component::Normal(_)))
}

/// Compare canonical existing ancestors, including cache/run paths not created yet.
fn resolved_location(path: &Path) -> Result<PathBuf, String> {
    let mut ancestor = path;
    let mut tail = Vec::new();
    while !ancestor.exists() {
        tail.push(
            ancestor
                .file_name()
                .ok_or("invalid storage path")?
                .to_owned(),
        );
        ancestor = ancestor.parent().ok_or("invalid storage path")?;
    }
    let mut resolved = fs::canonicalize(ancestor).map_err(|e| e.to_string())?;
    for part in tail.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

pub fn validate_storage(task: &Task, storage: &Path) -> Result<(), String> {
    let storage = resolved_location(storage)?;
    let cwd = fs::canonicalize(&task.cwd).map_err(|e| e.to_string())?;
    let policy = task.cache.as_ref().ok_or("missing cache policy")?;
    for relative in policy.inputs.iter().chain(&policy.outputs) {
        let path = cwd.join(relative);
        if storage.starts_with(&path) || path.starts_with(&storage) {
            return Err(format!(
                "cache/run storage overlaps a declared path: {}",
                relative.display()
            ));
        }
    }
    Ok(())
}

impl CachePolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.toolchain.is_empty() {
            return Err("cache policy requires a toolchain identity".into());
        }
        if self.outputs.is_empty() {
            return Err("cache policy requires declared outputs".into());
        }
        for path in self.inputs.iter().chain(&self.outputs) {
            if !safe_relative(path) {
                return Err(format!(
                    "cache paths must be relative without '.' or '..': {}",
                    path.display()
                ));
            }
        }
        for (i, output) in self.outputs.iter().enumerate() {
            for other in self.inputs.iter().chain(self.outputs.iter().take(i)) {
                if output.starts_with(other) || other.starts_with(output) {
                    return Err(format!(
                        "overlapping cache paths: {} and {}",
                        output.display(),
                        other.display()
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Reject symlink traversal and special files. Symlink artifacts will require an
/// explicit, contained-link representation rather than following host links.
fn checked_path(base: &Path, relative: &Path) -> Result<PathBuf, String> {
    if !safe_relative(relative) {
        return Err("unsafe artifact path".into());
    }
    let mut path = base.to_path_buf();
    for part in relative.components() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("symlinks are not cacheable: {}", path.display()))
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(path)
}
fn mode(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o777
    }
    #[cfg(not(unix))]
    {
        if meta.permissions().readonly() {
            0o444
        } else {
            0o644
        }
    }
}
fn set_mode(path: &Path, value: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(value & 0o777))
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    {
        let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        perms.set_readonly(value & 0o200 == 0);
        fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn snapshot(base: &Path, paths: &[PathBuf]) -> Result<Vec<Entry>, String> {
    fn visit(
        base: &Path,
        relative: &Path,
        entries: &mut BTreeMap<PathBuf, Entry>,
    ) -> Result<(), String> {
        let path = checked_path(base, relative)?;
        let meta = fs::symlink_metadata(&path)
            .map_err(|e| format!("declared path {}: {e}", path.display()))?;
        if !meta.is_dir() && !meta.is_file() {
            return Err(format!("unsupported artifact: {}", path.display()));
        }
        entries.insert(
            relative.into(),
            Entry {
                path: relative.into(),
                directory: meta.is_dir(),
                mode: mode(&meta),
                digest: if meta.is_file() {
                    Some(file_digest(&path)?)
                } else {
                    None
                },
            },
        );
        if meta.is_dir() {
            for item in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                visit(base, &relative.join(item.file_name()), entries)?;
            }
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    for path in paths {
        visit(base, path, &mut entries)?;
    }
    Ok(entries.into_values().collect())
}

/// Returns None when another process owns the same action. The scheduler can
/// retry without blocking unrelated tasks or losing cancellation responsiveness.
pub fn prepare(
    root: &Path,
    task: &Task,
    dependency_keys: &[String],
) -> Result<Option<Prepared>, String> {
    let policy = task.cache.as_ref().ok_or("missing cache policy")?;
    policy.validate()?;
    let executable = if Path::new(&task.program).is_absolute() {
        PathBuf::from(&task.program)
    } else if Path::new(&task.program).components().count() > 1 {
        task.cwd.join(&task.program)
    } else {
        task.env
            .get("PATH")
            .and_then(|path| {
                std::env::split_paths(path)
                    .map(|p| {
                        if p.is_absolute() {
                            p.join(&task.program)
                        } else {
                            task.cwd.join(p).join(&task.program)
                        }
                    })
                    .find(|p| {
                        fs::metadata(p).is_ok_and(|meta| meta.is_file() && mode(&meta) & 0o111 != 0)
                    })
            })
            .ok_or("cached tasks require an absolute program or explicit PATH")?
    };
    let mut components = BTreeMap::new();
    components.insert(
        "command".into(),
        encoded_digest(&(&task.program, &task.args))?,
    );
    components.insert("executable".into(), file_digest(&executable)?);
    components.insert(
        "executable_mode".into(),
        encoded_digest(&mode(
            &fs::metadata(&executable).map_err(|e| e.to_string())?,
        ))?,
    );
    components.insert(
        "inputs".into(),
        encoded_digest(&snapshot(&task.cwd, &policy.inputs)?)?,
    );
    components.insert("environment".into(), encoded_digest(&task.env)?);
    components.insert("toolchain".into(), digest(policy.toolchain.as_bytes()));
    components.insert("outputs".into(), encoded_digest(&policy.outputs)?);
    components.insert(
        "working_directory".into(),
        encoded_digest(&fs::canonicalize(&task.cwd).map_err(|e| e.to_string())?)?,
    );
    components.insert("dependencies".into(), encoded_digest(&dependency_keys)?);
    components.insert(
        "platform".into(),
        encoded_digest(&(std::env::consts::OS, std::env::consts::ARCH))?,
    );
    components.insert("cache_format".into(), digest(b"fluentci-action-v1"));
    let key = encoded_digest(&components)?;
    for dir in ["actions", "blobs", "locks"] {
        fs::create_dir_all(root.join(dir)).map_err(|e| e.to_string())?;
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks").join(&key))
        .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(error.to_string());
        }
    }
    #[cfg(not(unix))]
    {
        return Err("action caching currently requires Unix advisory locks".into());
    }
    Ok(Some(Prepared {
        key,
        executable,
        components,
        root: root.into(),
        _lock: lock,
    }))
}

impl Prepared {
    fn blob(&self, hash: &str) -> Result<PathBuf, String> {
        if !valid_digest(hash) {
            return Err("invalid blob digest".into());
        }
        Ok(self.root.join("blobs").join(hash))
    }
    fn store_blob(&self, source: &Path, expected: &str) -> Result<(), String> {
        let destination = self.blob(expected)?;
        if destination.exists() && file_digest(&destination)? == expected {
            return Ok(());
        }
        let mut staging =
            tempfile::NamedTempFile::new_in(self.root.join("blobs")).map_err(|e| e.to_string())?;
        let mut file = File::open(source).map_err(|e| e.to_string())?;
        std::io::copy(&mut file, &mut staging).map_err(|e| e.to_string())?;
        if file_digest(staging.path())? != expected {
            return Err(format!("file changed while caching: {}", source.display()));
        }
        staging.as_file().sync_all().map_err(|e| e.to_string())?;
        staging.persist(&destination).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Cacheable tasks start with clean declared outputs, so stale files cannot
    /// accidentally enter an action result. Inputs and outputs cannot overlap.
    pub fn clean_outputs(&self, task: &Task) -> Result<(), String> {
        for root in &task.cache.as_ref().ok_or("missing cache policy")?.outputs {
            let output = checked_path(&task.cwd, root)?;
            if output.is_dir() {
                fs::remove_dir_all(&output).map_err(|e| e.to_string())?;
            } else if output.exists() {
                fs::remove_file(&output).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn save(&self, task: &Task, stdout: &Path, stderr: &Path) -> Result<(), String> {
        let policy = task.cache.as_ref().ok_or("missing cache policy")?;
        if file_digest(&self.executable)? != self.components["executable"] {
            return Err("executable changed during execution; action was not cached".into());
        }
        // Never publish under the old key if declared inputs changed during execution.
        if encoded_digest(&snapshot(&task.cwd, &policy.inputs)?)? != self.components["inputs"] {
            return Err("declared inputs changed during execution; action was not cached".into());
        }
        let entries = snapshot(&task.cwd, &policy.outputs)?;
        for entry in &entries {
            if let Some(hash) = &entry.digest {
                self.store_blob(&checked_path(&task.cwd, &entry.path)?, hash)?;
            }
        }
        let stdout_digest = file_digest(stdout)?;
        let stderr_digest = file_digest(stderr)?;
        self.store_blob(stdout, &stdout_digest)?;
        self.store_blob(stderr, &stderr_digest)?;
        let manifest = Manifest {
            version: 1,
            key: self.key.clone(),
            outputs: entries,
            stdout: stdout_digest,
            stderr: stderr_digest,
        };
        let mut staging = tempfile::NamedTempFile::new_in(self.root.join("actions"))
            .map_err(|e| e.to_string())?;
        let record = ActionRecord {
            checksum: encoded_digest(&manifest)?,
            manifest,
        };
        serde_json::to_writer(&mut staging, &record).map_err(|e| e.to_string())?;
        staging.flush().map_err(|e| e.to_string())?;
        staging.as_file().sync_all().map_err(|e| e.to_string())?;
        staging
            .persist(self.root.join("actions").join(format!("{}.json", self.key)))
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Stage and validate every artifact before changing any workspace output.
    pub fn restore(&self, task: &Task, stdout: &Path, stderr: &Path) -> Result<bool, String> {
        let path = self.root.join("actions").join(format!("{}.json", self.key));
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(e.to_string()),
        };
        let record: ActionRecord =
            serde_json::from_slice(&bytes).map_err(|e| format!("invalid cache manifest: {e}"))?;
        if encoded_digest(&record.manifest)? != record.checksum {
            return Err("cache manifest checksum mismatch".into());
        }
        let manifest = record.manifest;
        if manifest.version != 1 || manifest.key != self.key {
            return Err("cache manifest identity mismatch".into());
        }
        let policy = task.cache.as_ref().ok_or("missing cache policy")?;
        let staging = tempfile::Builder::new()
            .prefix(".fluentci-restore-")
            .tempdir_in(&task.cwd)
            .map_err(|e| e.to_string())?;
        let artifacts = staging.path().join("artifacts");
        fs::create_dir(&artifacts).map_err(|e| e.to_string())?;
        let mut seen = std::collections::HashSet::new();
        for entry in &manifest.outputs {
            if !safe_relative(&entry.path)
                || !policy
                    .outputs
                    .iter()
                    .any(|root| entry.path.starts_with(root))
                || !seen.insert(entry.path.clone())
            {
                return Err("cache manifest contains an invalid output path".into());
            }
            checked_path(&task.cwd, &entry.path)?;
            let destination = artifacts.join(&entry.path);
            if entry.directory {
                if entry.digest.is_some() {
                    return Err("directory has a blob digest".into());
                }
                fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
            } else {
                let hash = entry.digest.as_ref().ok_or("file has no blob digest")?;
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::copy(self.blob(hash)?, &destination).map_err(|e| e.to_string())?;
                if file_digest(&destination)? != *hash {
                    return Err("cache blob failed integrity verification".into());
                }
                set_mode(&destination, entry.mode)?;
            }
        }
        for root in &policy.outputs {
            if !seen.contains(root) {
                return Err("cache manifest is missing an output root".into());
            }
            checked_path(&task.cwd, root)?;
        }
        // Verify logs too, before publishing restored outputs.
        let staged_stdout = staging.path().join(".stdout-log");
        let staged_stderr = staging.path().join(".stderr-log");
        for (hash, target) in [
            (&manifest.stdout, &staged_stdout),
            (&manifest.stderr, &staged_stderr),
        ] {
            fs::copy(self.blob(hash)?, target).map_err(|e| e.to_string())?;
            if file_digest(target)? != *hash {
                return Err("cached log failed integrity verification".into());
            }
        }
        for entry in manifest.outputs.iter().rev().filter(|e| e.directory) {
            set_mode(&artifacts.join(&entry.path), entry.mode)?;
        }
        for root in &policy.outputs {
            let destination = checked_path(&task.cwd, root)?;
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            if destination.is_dir() {
                fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
            } else if destination.exists() {
                fs::remove_file(&destination).map_err(|e| e.to_string())?;
            }
            fs::rename(artifacts.join(root), &destination).map_err(|e| e.to_string())?;
        }
        fs::copy(staged_stdout, stdout).map_err(|e| e.to_string())?;
        fs::copy(staged_stderr, stderr).map_err(|e| e.to_string())?;
        Ok(true)
    }
}
