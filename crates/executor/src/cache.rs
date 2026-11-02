//! Opt-in local action cache. Cacheability is a contract: all consumed inputs and
//! produced outputs must be declared. This is not a filesystem/network sandbox.
use crate::{Cancellation, Task};
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub path: PathBuf,
    pub directory: bool,
    pub mode: u32,
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
    cancel: Cancellation,
    // Process-scoped advisory lock; released on drop, including crashes.
    _lock: File,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn check_cancel(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("cache operation cancelled".into())
    } else {
        Ok(())
    }
}

fn copy_stream(
    source: &mut impl Read,
    destination: &mut impl Write,
    cancel: &Cancellation,
) -> Result<(), String> {
    let mut buffer = [0u8; 65536];
    loop {
        check_cancel(cancel)?;
        let count = source.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            return Ok(());
        }
        destination
            .write_all(&buffer[..count])
            .map_err(|e| e.to_string())?;
    }
}

pub(crate) fn copy_file(
    source: &Path,
    destination: &Path,
    cancel: &Cancellation,
) -> Result<(), String> {
    check_cancel(cancel)?;
    let mut input = File::open(source).map_err(|e| e.to_string())?;
    let mut output = File::create(destination).map_err(|e| e.to_string())?;
    copy_stream(&mut input, &mut output, cancel)?;
    fs::set_permissions(
        destination,
        input.metadata().map_err(|e| e.to_string())?.permissions(),
    )
    .map_err(|e| e.to_string())
}

fn file_digest(path: &Path, cancel: &Cancellation) -> Result<String, String> {
    check_cancel(cancel)?;
    let mut file = File::open(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        check_cancel(cancel)?;
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
pub(crate) fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|p| matches!(p, Component::Normal(_)))
}

/// Compare canonical existing ancestors, including cache/run paths not created yet.
pub(crate) fn resolved_location(path: &Path) -> Result<PathBuf, String> {
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

/// Declared writers must be ordered with every overlapping reader or writer.
/// Resource exclusion alone does not specify which version a reader consumes.
pub(crate) fn validate_workspace(
    tasks: &[Task],
    graph: &crate::graph::DependencyGraph,
) -> Result<(), String> {
    let mut paths: BTreeMap<PathBuf, Vec<(usize, bool)>> = BTreeMap::new();
    for (index, task) in tasks.iter().enumerate() {
        if let Some(policy) = &task.cache {
            let cwd = fs::canonicalize(&task.cwd).map_err(|e| e.to_string())?;
            for (declared, write) in [(&policy.inputs, false), (&policy.outputs, true)] {
                for relative in declared {
                    // Do not follow artifact symlinks into a different workspace.
                    let path = checked_path(&cwd, relative)?;
                    paths.entry(path).or_default().push((index, write));
                }
            }
        }
    }
    let mut ordered = std::collections::HashSet::new();
    for (path, accesses) in &paths {
        for ancestor in path.ancestors() {
            let Some(others) = paths.get(ancestor) else {
                continue;
            };
            // Avoid a quadratic reader/reader comparison for shared source trees.
            let writers: Vec<_> = others.iter().copied().filter(|(_, write)| *write).collect();
            for &(a, write_a) in accesses {
                let candidates = if write_a {
                    others.as_slice()
                } else {
                    writers.as_slice()
                };
                for &(b, _) in candidates {
                    if a == b {
                        continue;
                    }
                    let pair = (a.min(b), a.max(b));
                    if ordered.contains(&pair) {
                        continue;
                    }
                    let reaches = |from: usize, target: usize| {
                        let mut pending = vec![from];
                        let mut seen = std::collections::HashSet::new();
                        while let Some(node) = pending.pop() {
                            if node == target {
                                return true;
                            }
                            if seen.insert(node) {
                                pending.extend(graph.dependencies[node].iter().copied());
                            }
                        }
                        false
                    };
                    if !reaches(a, b) && !reaches(b, a) {
                        return Err(format!(
                            "unordered cache paths overlap between tasks {} and {} at {}; add a dependency or use separate output paths",
                            tasks[a].id, tasks[b].id, path.display()
                        ));
                    }
                    ordered.insert(pair);
                }
            }
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
pub(crate) fn checked_path(base: &Path, relative: &Path) -> Result<PathBuf, String> {
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
pub(crate) fn set_mode(path: &Path, value: u32) -> Result<(), String> {
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
pub(crate) fn snapshot(
    base: &Path,
    paths: &[PathBuf],
    cancel: &Cancellation,
) -> Result<Vec<Entry>, String> {
    fn visit(
        base: &Path,
        relative: &Path,
        entries: &mut BTreeMap<PathBuf, Entry>,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        check_cancel(cancel)?;
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
                    Some(file_digest(&path, cancel)?)
                } else {
                    None
                },
            },
        );
        if meta.is_dir() {
            for item in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                visit(base, &relative.join(item.file_name()), entries, cancel)?;
            }
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    for path in paths {
        visit(base, path, &mut entries, cancel)?;
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
    prepare_cancellable(root, task, dependency_keys, &Cancellation::default())
}

pub(crate) fn prepare_cancellable(
    root: &Path,
    task: &Task,
    dependency_keys: &[String],
    cancel: &Cancellation,
) -> Result<Option<Prepared>, String> {
    #[cfg(all(test, unix))]
    crate::cache_worker_tests::pause(&task.cwd, "prepare", cancel)?;
    check_cancel(cancel)?;
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
    components.insert("executable".into(), file_digest(&executable, cancel)?);
    components.insert(
        "executable_mode".into(),
        encoded_digest(&mode(
            &fs::metadata(&executable).map_err(|e| e.to_string())?,
        ))?,
    );
    components.insert(
        "inputs".into(),
        encoded_digest(&snapshot(&task.cwd, &policy.inputs, cancel)?)?,
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
        cancel: cancel.clone(),
        _lock: lock,
    }))
}

pub(crate) struct Lookup {
    pub action: Prepared,
    pub hit: bool,
    pub reason: String,
}

/// Run entirely on an owned worker while the scheduler retains resource leases.
pub(crate) fn lookup(
    root: &Path,
    task: &Task,
    dependencies: &[String],
    stdout: &Path,
    stderr: &Path,
    cancel: &Cancellation,
) -> Result<Option<Lookup>, String> {
    let Some(action) = prepare_cancellable(root, task, dependencies, cancel)? else {
        return Ok(None);
    };
    let (hit, reason) = match action.restore(task, stdout, stderr) {
        Ok(true) => (true, "verified cached outputs restored".into()),
        Ok(false) => (false, "no matching action".into()),
        Err(error) => {
            check_cancel(cancel)?;
            (false, format!("cache rejected: {error}"))
        }
    };
    check_cancel(cancel)?;
    if !hit {
        // Journal tailing is suspended for this task until lookup completes.
        let _ = fs::remove_file(stdout);
        let _ = fs::remove_file(stderr);
        action.clean_outputs(task)?;
    }
    Ok(Some(Lookup {
        action,
        hit,
        reason,
    }))
}

impl Prepared {
    pub(crate) fn with_cancellation(mut self, cancel: Cancellation) -> Self {
        self.cancel = cancel;
        self
    }

    fn blob(&self, hash: &str) -> Result<PathBuf, String> {
        if !valid_digest(hash) {
            return Err("invalid blob digest".into());
        }
        Ok(self.root.join("blobs").join(hash))
    }
    fn store_blob(&self, source: &Path, expected: &str) -> Result<(), String> {
        let destination = self.blob(expected)?;
        if destination.exists() && file_digest(&destination, &self.cancel)? == expected {
            return Ok(());
        }
        let mut staging =
            tempfile::NamedTempFile::new_in(self.root.join("blobs")).map_err(|e| e.to_string())?;
        let mut file = File::open(source).map_err(|e| e.to_string())?;
        copy_stream(&mut file, &mut staging, &self.cancel)?;
        if file_digest(staging.path(), &self.cancel)? != expected {
            return Err(format!("file changed while caching: {}", source.display()));
        }
        staging.as_file().sync_all().map_err(|e| e.to_string())?;
        staging.persist(&destination).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Cacheable tasks start with clean declared outputs, so stale files cannot
    /// accidentally enter an action result. Inputs and outputs cannot overlap.
    pub fn clean_outputs(&self, task: &Task) -> Result<(), String> {
        check_cancel(&self.cancel)?;
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
        #[cfg(all(test, unix))]
        crate::cache_worker_tests::pause_save(&task.cwd, &self.cancel)?;
        let policy = task.cache.as_ref().ok_or("missing cache policy")?;
        if file_digest(&self.executable, &self.cancel)? != self.components["executable"] {
            return Err("executable changed during execution; action was not cached".into());
        }
        // Never publish under the old key if declared inputs changed during execution.
        if encoded_digest(&snapshot(&task.cwd, &policy.inputs, &self.cancel)?)?
            != self.components["inputs"]
        {
            return Err("declared inputs changed during execution; action was not cached".into());
        }
        let entries = snapshot(&task.cwd, &policy.outputs, &self.cancel)?;
        for entry in &entries {
            if let Some(hash) = &entry.digest {
                self.store_blob(&checked_path(&task.cwd, &entry.path)?, hash)?;
            }
        }
        let stdout_digest = file_digest(stdout, &self.cancel)?;
        let stderr_digest = file_digest(stderr, &self.cancel)?;
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
        check_cancel(&self.cancel)?;
        staging
            .persist(self.root.join("actions").join(format!("{}.json", self.key)))
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Stage and validate every artifact before changing any workspace output.
    pub fn restore(&self, task: &Task, stdout: &Path, stderr: &Path) -> Result<bool, String> {
        #[cfg(all(test, unix))]
        crate::cache_worker_tests::pause(&task.cwd, "restore", &self.cancel)?;
        check_cancel(&self.cancel)?;
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
                copy_file(&self.blob(hash)?, &destination, &self.cancel)?;
                if file_digest(&destination, &self.cancel)? != *hash {
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
            copy_file(&self.blob(hash)?, target, &self.cancel)?;
            if file_digest(target, &self.cancel)? != *hash {
                return Err("cached log failed integrity verification".into());
            }
        }
        for entry in manifest.outputs.iter().rev().filter(|e| e.directory) {
            set_mode(&artifacts.join(&entry.path), entry.mode)?;
        }
        // A log write failure must not occur after workspace publication.
        copy_file(&staged_stdout, stdout, &self.cancel)?;
        copy_file(&staged_stderr, stderr, &self.cancel)?;
        check_cancel(&self.cancel)?;
        publish_outputs(&task.cwd, &artifacts, &policy.outputs)?;
        Ok(true)
    }
}

/// Preserve existing outputs until every replacement succeeds. This rolls back
/// ordinary filesystem errors; it is not a crash-atomic multi-path transaction.
fn publish_outputs(cwd: &Path, artifacts: &Path, outputs: &[PathBuf]) -> Result<(), String> {
    let backups = tempfile::Builder::new()
        .prefix(".fluentci-backup-")
        .tempdir_in(cwd)
        .map_err(|e| e.to_string())?;
    let mut replaced = Vec::new();
    let publication = (|| {
        for (index, root) in outputs.iter().enumerate() {
            let destination = checked_path(cwd, root)?;
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let backup = backups.path().join(index.to_string());
            let existed = destination.try_exists().map_err(|e| e.to_string())?;
            if existed {
                fs::rename(&destination, &backup).map_err(|e| e.to_string())?;
            }
            // Record the backup before attempting the replacement, so even a
            // failure on this rename restores the original output.
            replaced.push((destination.clone(), backup, existed, false));
            fs::rename(artifacts.join(root), &destination).map_err(|e| e.to_string())?;
            replaced.last_mut().unwrap().3 = true;
        }
        Ok::<_, String>(())
    })();
    if let Err(error) = publication {
        let mut failures = Vec::new();
        for (destination, backup, existed, installed) in replaced.into_iter().rev() {
            let rollback = (|| {
                if installed {
                    // Move back instead of recursively deleting: read-only
                    // output trees can still be rolled back by their parent.
                    let relative = destination.strip_prefix(cwd).map_err(|e| e.to_string())?;
                    fs::rename(&destination, artifacts.join(relative))
                        .map_err(|e| e.to_string())?;
                }
                if existed {
                    fs::rename(&backup, &destination).map_err(|e| e.to_string())?;
                }
                Ok::<_, String>(())
            })();
            if let Err(reason) = rollback {
                failures.push(format!("{}: {reason}", destination.display()));
            }
        }
        if !failures.is_empty() {
            let retained = backups.into_path();
            return Err(format!(
                "cache publication failed: {error}; rollback failed: {}; backups retained at {}",
                failures.join("; "),
                retained.display()
            ));
        }
        return Err(format!(
            "cache publication failed (outputs rolled back): {error}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod publication_tests {
    use super::*;

    #[test]
    fn later_failure_restores_replaced_outputs_and_preserves_blocker() {
        let workspace = tempfile::tempdir().unwrap();
        let staged = tempfile::tempdir().unwrap();
        fs::write(workspace.path().join("first"), "original").unwrap();
        fs::write(workspace.path().join("blocked"), "keep").unwrap();
        fs::write(staged.path().join("first"), "cached").unwrap();
        fs::create_dir(staged.path().join("blocked")).unwrap();
        fs::write(staged.path().join("blocked/second"), "second").unwrap();
        let error = publish_outputs(
            workspace.path(),
            staged.path(),
            &["first".into(), "blocked/second".into()],
        )
        .unwrap_err();
        assert!(error.contains("outputs rolled back"), "{error}");
        assert_eq!(
            fs::read_to_string(workspace.path().join("first")).unwrap(),
            "original"
        );
        assert_eq!(
            fs::read_to_string(workspace.path().join("blocked")).unwrap(),
            "keep"
        );
        assert_eq!(
            fs::read_to_string(staged.path().join("first")).unwrap(),
            "cached"
        );
    }

    #[test]
    fn failed_replacement_restores_its_backup_and_removes_new_outputs() {
        let workspace = tempfile::tempdir().unwrap();
        let staged = tempfile::tempdir().unwrap();
        fs::write(workspace.path().join("second"), "original").unwrap();
        fs::write(staged.path().join("first"), "cached").unwrap();
        // The second staged output is deliberately missing.
        assert!(publish_outputs(
            workspace.path(),
            staged.path(),
            &["first".into(), "second".into()]
        )
        .is_err());
        assert!(!workspace.path().join("first").exists());
        assert_eq!(
            fs::read_to_string(workspace.path().join("second")).unwrap(),
            "original"
        );
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;

    #[test]
    fn streaming_copy_stops_between_chunks() {
        struct CancelAfterRead {
            cancel: Cancellation,
            reads: usize,
        }
        impl Read for CancelAfterRead {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                self.reads += 1;
                buffer.fill(42);
                self.cancel.cancel();
                Ok(buffer.len())
            }
        }
        let cancel = Cancellation::default();
        let mut source = CancelAfterRead {
            cancel: cancel.clone(),
            reads: 0,
        };
        let mut destination = Vec::new();
        assert!(copy_stream(&mut source, &mut destination, &cancel)
            .unwrap_err()
            .contains("cancelled"));
        assert_eq!(source.reads, 1);
        assert_eq!(destination.len(), 65536);
    }

    #[cfg(unix)]
    #[test]
    fn cancelled_cache_work_does_not_publish_or_replace_outputs() {
        let root = tempfile::tempdir().unwrap();
        let task: Task = serde_json::from_value(serde_json::json!({
            "id": "build", "program": "/bin/sh", "cwd": root.path(),
            "cache": {"inputs": [], "outputs": ["output"], "toolchain": "test"}
        }))
        .unwrap();
        let cache = root.path().join("cache");
        let output = root.path().join("output");
        let stdout = root.path().join("stdout");
        let stderr = root.path().join("stderr");
        fs::write(&output, "cached").unwrap();
        fs::write(&stdout, "log").unwrap();
        fs::write(&stderr, "").unwrap();
        let cancel = Cancellation::default();
        let prepared = prepare_cancellable(&cache, &task, &[], &cancel)
            .unwrap()
            .unwrap();
        cancel.cancel();
        assert!(prepared
            .save(&task, &stdout, &stderr)
            .unwrap_err()
            .contains("cancelled"));
        assert!(!cache
            .join("actions")
            .join(format!("{}.json", prepared.key))
            .exists());
        drop(prepared);
        let cancel = Cancellation::default();
        let prepared = prepare_cancellable(&cache, &task, &[], &cancel)
            .unwrap()
            .unwrap();
        prepared.save(&task, &stdout, &stderr).unwrap();
        fs::write(&output, "preserved").unwrap();
        cancel.cancel();
        assert!(prepared
            .restore(&task, &stdout, &stderr)
            .unwrap_err()
            .contains("cancelled"));
        assert!(prepared
            .clean_outputs(&task)
            .unwrap_err()
            .contains("cancelled"));
        assert_eq!(fs::read_to_string(output).unwrap(), "preserved");
        assert!(
            matches!(prepare_cancellable(&root.path().join("unused"), &task, &[], &cancel), Err(e) if e.contains("cancelled"))
        );
        assert!(!root.path().join("unused").exists());
    }
}
