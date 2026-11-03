//! Independent, authenticated run API. No mutable current-pipeline state.
mod admission;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use fluentci_executor::{Cancellation, Plan};
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    thread::JoinHandle,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
struct RunState {
    #[serde(default)]
    created_at_ms: u64,
    id: String,
    status: String,
    error: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Submission {
    run_id: String,
}

// Serialize cancellation with the queued-to-running transition. A queued
// cancellation is durable before acknowledgement and cannot be started afterward.
struct RunControl {
    cancel: Cancellation,
    path: PathBuf,
    state: Mutex<RunState>,
}
impl RunControl {
    fn cancel(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        // Even if persistence fails, prevent execution in this server process.
        self.cancel.cancel();
        if state.status == "queued" {
            let mut cancelled = state.clone();
            cancelled.status = "cancelled".into();
            cancelled.error = Some("run cancelled before execution".into());
            save(&self.path, &cancelled)?;
            *state = cancelled;
        }
        Ok(())
    }
    fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if self.cancel.is_cancelled() {
            return Err("run cancelled before execution".into());
        }
        if state.status == "queued" {
            let mut running = state.clone();
            running.status = "running".into();
            save(&self.path, &running)?;
            *state = running;
        }
        Ok(())
    }
    fn finish(&self, status: &str, error: Option<String>) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if state.status == "cancelled" {
            return Ok(());
        }
        let mut terminal = state.clone();
        terminal.status = status.into();
        terminal.error = error;
        save(&self.path, &terminal)?;
        *state = terminal;
        Ok(())
    }
}
struct Active {
    control: Arc<RunControl>,
    worker: JoinHandle<()>,
}
pub struct Runs {
    _store_lock: fs::File,
    root: PathBuf,
    token: String,
    admission: std::sync::Arc<admission::Admission>,
    active: Mutex<HashMap<String, Active>>,
    closing: AtomicBool,
    shutdown_lock: Mutex<()>,
    history_index: RwLock<BTreeSet<(u64, String)>>,
}
fn save(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut file =
        tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, value).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    fs::File::open(path.parent().unwrap())
        .and_then(|dir| dir.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
// A torn, malformed, or incomplete report is not evidence of completion. Leave
// it on disk for diagnosis, but do not let it prevent unrelated runs recovering.
fn recovered_outcome(path: &Path) -> Option<&'static str> {
    let plan: Plan = serde_json::from_slice(&fs::read(path.join("plan.json")).ok()?).ok()?;
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(path.join("execution/result.json")).ok()?).ok()?;
    let tasks = report.get("tasks")?.as_array()?;
    if tasks.len() != plan.tasks.len() {
        return None;
    }
    let mut success = true;
    for (task, expected) in tasks.iter().zip(&plan.tasks) {
        if task.get("id")?.as_str()? != expected.id {
            return None;
        }
        match task.get("status")?.as_str()? {
            "succeeded" => {}
            "failed" | "skipped" | "cancelled" | "timed_out" => success = false,
            _ => return None,
        }
    }
    match report.get("outcome") {
        None => Some(if success { "succeeded" } else { "failed" }),
        Some(value) => match value.as_str()? {
            "succeeded" if success => Some("succeeded"),
            "failed" if !success => Some("failed"),
            "cancelled" if !success => Some("cancelled"),
            _ => None,
        },
    }
}

impl Runs {
    pub fn new(root: &Path, token: String, max_runs: usize) -> Result<Self, String> {
        Self::with_queue(root, token, max_runs, 0)
    }
    pub fn with_queue(
        root: &Path,
        token: String,
        max_runs: usize,
        max_queued: usize,
    ) -> Result<Self, String> {
        if token.len() < 16 || max_runs == 0 {
            return Err(
                "run server requires a token of at least 16 bytes and positive max-runs".into(),
            );
        }
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(root).map_err(|e| e.to_string())?;
        let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
        let mut options = fs::OpenOptions::new();
        options.create(true).truncate(false).read(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let store_lock = options
            .open(root.join(".lock"))
            .map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            if unsafe { libc::flock(store_lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
                return Err("another run server owns this data directory".into());
            }
        }
        let submissions = root.join("submissions");
        let mut index = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            index.mode(0o700);
        }
        if !submissions.exists() {
            index.create(&submissions).map_err(|e| e.to_string())?;
        }
        #[cfg(unix)]
        fs::File::open(&root)
            .and_then(|dir| dir.sync_all())
            .map_err(|e| e.to_string())?;
        // A previous process cannot own live runs in this store. Never silently
        // report them as running after a restart.
        let mut history_index = BTreeSet::new();
        for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if Uuid::parse_str(&entry.file_name().to_string_lossy()).is_err()
                || !entry.file_type().map_err(|e| e.to_string())?.is_dir()
            {
                continue;
            }
            let path = entry.path().join("state.json");
            if let Ok(bytes) = fs::read(&path) {
                let mut state: RunState =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if state.id != entry.file_name().to_string_lossy() {
                    return Err("run state identity mismatch".into());
                }
                history_index.insert((state.created_at_ms, state.id.clone()));
                if state.status == "queued" {
                    state.status = "interrupted".into();
                    state.error = Some(
                        "server stopped before queued run execution; submit a new run explicitly"
                            .into(),
                    );
                    save(&path, &state)?;
                } else if state.status == "running" {
                    match recovered_outcome(&entry.path()) {
                        Some(outcome) => {
                            state.status = outcome.into();
                            state.error = None;
                        }
                        None => {
                            state.status = "interrupted".into();
                            state.error = Some(
                                "server stopped without a complete result matching the saved plan"
                                    .into(),
                            );
                        }
                    }
                    save(&path, &state)?;
                }
            }
        }
        Ok(Self {
            _store_lock: store_lock,
            root,
            token,
            admission: admission::Admission::new(max_runs, max_queued),
            active: Mutex::new(HashMap::new()),
            closing: AtomicBool::new(false),
            shutdown_lock: Mutex::new(()),
            history_index: RwLock::new(history_index),
        })
    }
    fn authorized(&self, request: &HttpRequest) -> bool {
        let Some(token) = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
        else {
            return false;
        };
        token.len() == self.token.len()
            && token
                .bytes()
                .zip(self.token.bytes())
                .fold(0u8, |diff, (a, b)| diff | (a ^ b))
                == 0
    }
    fn path(&self, id: &str) -> Result<PathBuf, HttpResponse> {
        if Uuid::parse_str(id).is_err() {
            return Err(HttpResponse::NotFound().finish());
        }
        let path = self.root.join(id);
        if !path.is_dir() {
            return Err(HttpResponse::NotFound().finish());
        }
        Ok(path)
    }
    pub fn shutdown(&self) {
        // Serialize shutdown callers so each returns only after cleanup finishes.
        let _shutdown = self.shutdown_lock.lock().unwrap();
        let active = {
            let mut active = self.active.lock().unwrap();
            // Admission checks this under the same lock before registering a worker.
            self.closing.store(true, Ordering::Release);
            std::mem::take(&mut *active)
        };
        for run in active.values() {
            if let Err(error) = run.control.cancel() {
                eprintln!("could not persist queued cancellation: {error}");
            }
        }
        for (_, run) in active {
            let _ = run.worker.join();
        }
    }
}
impl Drop for Runs {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn submission_path(runs: &Runs, key: &str) -> PathBuf {
    let digest = Sha256::digest(key.as_bytes());
    let name: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    runs.root.join("submissions").join(format!("{name}.json"))
}

fn replay(runs: &Runs, key: Option<&str>, plan: &Plan) -> Result<Option<RunState>, (u16, String)> {
    let Some(key) = key else {
        return Ok(None);
    };
    let bytes = match fs::read(submission_path(runs, key)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err((500, "could not read submission record".into())),
    };
    let stored = (|| {
        let record: Submission = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if Uuid::parse_str(&record.run_id).is_err() {
            return Err("invalid stored run ID".into());
        }
        let root = runs.root.join(&record.run_id);
        let original: Plan =
            serde_json::from_slice(&fs::read(root.join("plan.json")).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let state: RunState =
            serde_json::from_slice(&fs::read(root.join("state.json")).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let matches = serde_json::to_value(original).map_err(|e| e.to_string())?
            == serde_json::to_value(plan).map_err(|e| e.to_string())?;
        Ok::<_, String>((matches, state))
    })()
    .map_err(|_| (500, "could not recover recorded submission".into()))?;
    if !stored.0 {
        return Err((
            409,
            "Idempotency-Key was already used with a different plan".into(),
        ));
    }
    Ok(Some(stored.1))
}

async fn submit(
    request: HttpRequest,
    runs: web::Data<Runs>,
    plan: web::Json<Plan>,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
    if request.headers().get_all("Idempotency-Key").count() > 1 {
        return HttpResponse::BadRequest().body("provide one Idempotency-Key header");
    }
    let key =
        match request.headers().get("Idempotency-Key") {
            None => None,
            Some(value) => match value.to_str() {
                Ok(value)
                    if !value.is_empty()
                        && value.len() <= 128
                        && value
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')) =>
                {
                    Some(value.to_owned())
                }
                _ => return HttpResponse::BadRequest().body(
                    "Idempotency-Key requires 1-128 ASCII letters, digits, hyphens, or underscores",
                ),
            },
        };
    let plan = plan.into_inner();
    if plan.inherit_env {
        return HttpResponse::BadRequest().body("server plans require inherit_env: false");
    }
    let headers: HashMap<String, String> = ["traceparent", "tracestate"]
        .into_iter()
        .filter_map(|key| {
            request
                .headers()
                .get(key)
                .and_then(|v| v.to_str().ok())
                .map(|v| (key.into(), v.into()))
        })
        .collect();
    let parent = TraceContextPropagator::new().extract(&headers);
    let admission_key = key.clone();
    match tokio::task::spawn_blocking(move || admit(&runs, plan, parent, admission_key.as_deref()))
        .await
    {
        Ok(Ok(state)) => {
            let mut response = HttpResponse::Accepted();
            if let Some(key) = key {
                response.insert_header(("Idempotency-Key", key));
            }
            response
                .insert_header(("Location", format!("/v1/runs/{}", state.id)))
                .json(state)
        }
        Ok(Err((status, error))) => {
            let mut response =
                HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap());
            if status == 429 {
                response.insert_header(("Retry-After", "1"));
            }
            response.body(error)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
fn admit(
    runs: &Runs,
    plan: Plan,
    parent: opentelemetry::Context,
    key: Option<&str>,
) -> Result<RunState, (u16, String)> {
    if runs.closing.load(Ordering::Acquire) {
        return Err((503, "run server is shutting down".into()));
    }
    if plan.jobs > 64 || plan.max_services > 32 || plan.tasks.len() > 10_000 {
        return Err((
            400,
            "plan exceeds server limits (64 jobs, 32 services, 10000 tasks)".into(),
        ));
    }
    if let Some(existing) = replay(runs, key, &plan)? {
        return Ok(existing);
    }
    if let Err(error) = plan.validate() {
        return Err((400, error));
    }
    let mut active = runs.active.lock().unwrap();
    if runs.closing.load(Ordering::Acquire) {
        return Err((503, "run server is shutting down".into()));
    }
    // Validation ran outside the admission lock; another request may have won.
    if let Some(existing) = replay(runs, key, &plan)? {
        return Ok(existing);
    }
    active.retain(|_, run| !run.worker.is_finished());
    let id = Uuid::new_v4().to_string();
    let mut reservation = runs
        .admission
        .reserve(id.clone())
        .ok_or_else(|| (429, "run and queue capacity reached".to_owned()))?;
    let queued = reservation.queued();
    let path = runs.root.join(&id);
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| (500, "system clock is before Unix epoch".into()))?
        .as_millis()
        .min(u64::MAX as u128) as u64;
    let state = RunState {
        created_at_ms,
        id: id.clone(),
        status: if queued { "queued" } else { "running" }.into(),
        error: None,
    };
    let preparation = (|| {
        let mut dir = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            dir.mode(0o700);
        }
        dir.create(&path).map_err(|e| e.to_string())?;
        save(&path.join("plan.json"), &plan)?;
        save(&path.join("state.json"), &state)?;
        runs.history_index
            .write()
            .unwrap()
            .insert((created_at_ms, id.clone()));
        #[cfg(unix)]
        fs::File::open(&runs.root)
            .and_then(|dir| dir.sync_all())
            .map_err(|e| e.to_string())?;
        // Persist the association before executing any command. A restart replays
        // interrupted state instead of re-running possibly completed side effects.
        if let Some(key) = key {
            save(
                &submission_path(runs, key),
                &Submission { run_id: id.clone() },
            )?;
        }
        Ok::<_, String>(())
    })();
    if preparation.is_err() {
        let persisted = save(
            &path.join("state.json"),
            &RunState {
                created_at_ms,
                id: id.clone(),
                status: "failed".into(),
                error: Some("could not persist run".into()),
            },
        );
        if persisted.is_ok() {
            runs.history_index
                .write()
                .unwrap()
                .insert((created_at_ms, id.clone()));
        }
        return Err((500, "could not persist run".into()));
    }
    let cancel = Cancellation::default();
    let worker_cancel = cancel.clone();
    let worker_path = path.clone();
    let control = Arc::new(RunControl {
        cancel,
        path: path.join("state.json"),
        state: Mutex::new(state.clone()),
    });
    let worker_control = control.clone();
    let worker = std::thread::Builder::new()
        .name(format!("run-{id}"))
        .spawn(move || {
            let _guard = parent.attach();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                reservation.wait(&worker_cancel)?;
                worker_control.start()?;
                plan.execute(&worker_path.join("execution"), &worker_cancel)
            }));
            let (status, error) = match outcome {
                Ok(Ok(result)) => (result.outcome.as_str(), None),
                Ok(Err(error)) if worker_cancel.is_cancelled() => ("cancelled", Some(error)),
                Ok(Err(error)) => ("failed", Some(error)),
                Err(_) => ("failed", Some("executor panicked".into())),
            };
            if let Err(error) = worker_control.finish(status, error) {
                eprintln!("could not persist run completion: {error}");
            }
        });
    match worker {
        Ok(worker) => {
            active.insert(id.clone(), Active { control, worker });
            Ok(state)
        }
        Err(_) => {
            let _ = save(
                &path.join("state.json"),
                &RunState {
                    created_at_ms,
                    id,
                    status: "failed".into(),
                    error: Some("could not start run worker".into()),
                },
            );
            Err((500, "could not start run worker".into()))
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryQuery {
    #[serde(default = "history_limit")]
    limit: usize,
    before: Option<String>,
}
fn history_limit() -> usize {
    50
}
#[derive(Serialize)]
struct RunPage {
    runs: Vec<RunState>,
    next_cursor: Option<String>,
}
fn history_page(
    store: &Runs,
    limit: usize,
    before: Option<(u64, String)>,
) -> Result<RunPage, String> {
    // Copy only this page's keys, then release the index lock before filesystem I/O.
    let keys: Vec<_> = {
        let index = store.history_index.read().unwrap();
        match before {
            Some(before) => index
                .range(..before)
                .rev()
                .take(limit + 1)
                .cloned()
                .collect(),
            None => index.iter().rev().take(limit + 1).cloned().collect(),
        }
    };
    let more = keys.len() > limit;
    let mut runs = Vec::with_capacity(limit.min(keys.len()));
    for (created_at_ms, id) in keys.into_iter().take(limit) {
        let bytes = fs::read(store.root.join(&id).join("state.json")).map_err(|e| e.to_string())?;
        let state: RunState = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if state.id != id || state.created_at_ms != created_at_ms {
            return Err("run state history identity mismatch".into());
        }
        runs.push(state);
    }
    let next_cursor = if more {
        runs.last()
            .map(|run| format!("{}_{}", run.created_at_ms, run.id))
    } else {
        None
    };
    Ok(RunPage { runs, next_cursor })
}
async fn history(
    request: HttpRequest,
    runs: web::Data<Runs>,
    query: web::Query<HistoryQuery>,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
    if !(1..=200).contains(&query.limit) {
        return HttpResponse::BadRequest().body("limit must be 1-200");
    }
    let before = match query.before.as_deref() {
        None => None,
        Some(cursor) => match cursor.split_once('_').and_then(|(time, id)| {
            let time = time.parse::<u64>().ok()?;
            let parsed = Uuid::parse_str(id).ok()?;
            (parsed.to_string() == id).then(|| (time, id.to_owned()))
        }) {
            Some(before) => Some(before),
            None => return HttpResponse::BadRequest().body("invalid history cursor"),
        },
    };
    let limit = query.limit;
    match tokio::task::spawn_blocking(move || history_page(&runs, limit, before)).await {
        Ok(Ok(page)) => HttpResponse::Ok().json(page),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

async fn state(request: HttpRequest, runs: web::Data<Runs>, id: web::Path<String>) -> HttpResponse {
    read_file(request, runs, id, "state.json").await
}
async fn result(
    request: HttpRequest,
    runs: web::Data<Runs>,
    id: web::Path<String>,
) -> HttpResponse {
    read_file(request, runs, id, "execution/result.json").await
}
async fn read_file(
    request: HttpRequest,
    runs: web::Data<Runs>,
    id: web::Path<String>,
    file: &str,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
    let path = match runs.path(&id) {
        Ok(path) => path,
        Err(response) => return response,
    };
    match tokio::fs::read(path.join(file)).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("application/json")
            .body(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
async fn cancel(
    request: HttpRequest,
    runs: web::Data<Runs>,
    id: web::Path<String>,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
    if let Err(response) = runs.path(&id) {
        return response;
    }
    let id = id.into_inner();
    match tokio::task::spawn_blocking(move || {
        // Shutdown drains the active map before cancelling and joining workers.
        // Wait for it rather than acknowledging a run it has not yet cancelled.
        let _shutdown = runs.shutdown_lock.lock().unwrap();
        let control = runs
            .active
            .lock()
            .unwrap()
            .get(&id)
            .map(|run| run.control.clone());
        match control {
            Some(control) => control.cancel(),
            None => Ok(()),
        }
    })
    .await
    {
        Ok(Ok(())) => HttpResponse::Accepted().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}
#[derive(Deserialize)]
struct Cursor {
    #[serde(default)]
    cursor: u64,
}
async fn events(
    request: HttpRequest,
    runs: web::Data<Runs>,
    id: web::Path<String>,
    cursor: web::Query<Cursor>,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
    let path = match runs.path(&id) {
        Ok(path) => path,
        Err(response) => return response,
    };
    let read = async {
        let mut file = tokio::fs::File::open(path.join("execution/events.jsonl")).await?;
        if cursor.cursor > file.metadata().await?.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cursor beyond journal",
            ));
        }
        if cursor.cursor > 0 {
            file.seek(std::io::SeekFrom::Start(cursor.cursor - 1))
                .await?;
            let mut previous = [0u8; 1];
            file.read_exact(&mut previous).await?;
            if previous[0] != b'\n' {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "cursor is not an event boundary",
                ));
            }
        }
        file.seek(std::io::SeekFrom::Start(cursor.cursor)).await?;
        let mut bytes = Vec::new();
        file.take(4 * 1024 * 1024).read_to_end(&mut bytes).await?;
        let length = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |n| n + 1);
        bytes.truncate(length);
        Ok::<_, std::io::Error>(bytes)
    }
    .await;
    match read {
        Ok(bytes) => HttpResponse::Ok()
            .insert_header((
                "X-Next-Cursor",
                (cursor.cursor + bytes.len() as u64).to_string(),
            ))
            .content_type("application/x-ndjson")
            .body(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HttpResponse::Ok()
            .insert_header(("X-Next-Cursor", cursor.cursor.to_string()))
            .content_type("application/x-ndjson")
            .finish(),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {
            HttpResponse::BadRequest().body("invalid journal cursor")
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
async fn logs(
    request: HttpRequest,
    runs: web::Data<Runs>,
    route: web::Path<(String, usize, String)>,
    cursor: web::Query<Cursor>,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
    let (id, index, stream) = route.into_inner();
    if index >= 10_000
        || !matches!(
            stream.as_str(),
            "stdout" | "stderr" | "probe_stdout" | "probe_stderr"
        )
    {
        return HttpResponse::NotFound().finish();
    }
    let path = match runs.path(&id) {
        Ok(path) => path,
        Err(response) => return response,
    };
    let read = async {
        let mut file =
            tokio::fs::File::open(path.join("execution").join(format!("{index}.{stream}"))).await?;
        if cursor.cursor > file.metadata().await?.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cursor beyond log",
            ));
        }
        file.seek(std::io::SeekFrom::Start(cursor.cursor)).await?;
        let mut bytes = Vec::new();
        file.take(1024 * 1024).read_to_end(&mut bytes).await?;
        Ok::<_, std::io::Error>(bytes)
    }
    .await;
    match read {
        Ok(bytes) => HttpResponse::Ok()
            .insert_header((
                "X-Next-Cursor",
                (cursor.cursor + bytes.len() as u64).to_string(),
            ))
            .content_type("application/octet-stream")
            .body(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HttpResponse::NotFound().finish(),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {
            HttpResponse::BadRequest().body("invalid log cursor")
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
pub fn configure(config: &mut web::ServiceConfig) {
    config
        .app_data(web::JsonConfig::default().limit(1024 * 1024))
        .route("/v1/runs", web::post().to(submit))
        .route("/v1/runs", web::get().to(history))
        .route("/v1/runs/{id}", web::get().to(state))
        .route("/v1/runs/{id}/result", web::get().to(result))
        .route("/v1/runs/{id}/cancel", web::post().to(cancel))
        .route("/v1/runs/{id}/events", web::get().to(events))
        .route("/v1/runs/{id}/logs/{index}/{stream}", web::get().to(logs));
}
pub async fn start(listen: &str, root: &Path, max_runs: usize) -> anyhow::Result<()> {
    start_with_queue(listen, root, max_runs, 0).await
}

pub async fn start_with_queue(
    listen: &str,
    root: &Path,
    max_runs: usize,
    max_queued: usize,
) -> anyhow::Result<()> {
    let token = std::env::var("FLUENTCI_RUN_TOKEN")
        .map_err(|_| anyhow::anyhow!("set FLUENTCI_RUN_TOKEN to authenticate run requests"))?;
    let runs = web::Data::new(
        Runs::with_queue(root, token, max_runs, max_queued).map_err(anyhow::Error::msg)?,
    );
    fluentci_core::init_tracer().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let app_runs = runs.clone();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::DefaultHeaders::new().add(("Cache-Control", "no-store")))
            .app_data(app_runs.clone())
            .configure(configure)
    })
    .bind(listen)?
    .run();
    let outcome = server.await;
    tokio::task::spawn_blocking(move || {
        runs.shutdown();
        fluentci_core::shutdown_tracer();
    })
    .await?;
    outcome.map_err(Into::into)
}

#[cfg(all(test, unix))]
mod lifecycle_tests {
    use super::*;
    use std::sync::Barrier;

    fn plan(root: &Path) -> Plan {
        serde_json::from_value(serde_json::json!({
            "inherit_env": false,
            "tasks": [{"id":"work", "program":"/bin/sh",
                "args":["-c", "while :; do /bin/sleep 0.02; done"],
                "cwd":root, "timeout_ms":5000}]
        }))
        .unwrap()
    }

    fn queued_control(root: &Path) -> RunControl {
        let state = RunState {
            created_at_ms: 1,
            id: Uuid::new_v4().to_string(),
            status: "queued".into(),
            error: None,
        };
        let path = root.join("state.json");
        save(&path, &state).unwrap();
        RunControl {
            cancel: Cancellation::default(),
            path,
            state: Mutex::new(state),
        }
    }

    #[test]
    fn queued_cancellation_is_durable_without_worker_and_prevents_start() {
        let root = tempfile::tempdir().unwrap();
        let control = queued_control(root.path());
        control.cancel().unwrap();
        let state: RunState = serde_json::from_slice(&fs::read(&control.path).unwrap()).unwrap();
        assert_eq!(state.status, "cancelled");
        assert!(control.start().is_err());
        // Neither completion nor repeated cancellation may undo the durable result.
        control
            .finish("failed", Some("late worker failure".into()))
            .unwrap();
        control.cancel().unwrap();
        let state: RunState = serde_json::from_slice(&fs::read(&control.path).unwrap()).unwrap();
        assert_eq!(state.status, "cancelled");
        assert_eq!(
            state.error.as_deref(),
            Some("run cancelled before execution")
        );
    }

    #[test]
    fn failed_cancellation_persistence_is_reported_and_prevents_start() {
        let root = tempfile::tempdir().unwrap();
        let control = queued_control(root.path());
        fs::remove_file(&control.path).unwrap();
        fs::create_dir(&control.path).unwrap();
        assert!(control.cancel().is_err());
        assert!(control.start().is_err());
        fs::remove_dir(&control.path).unwrap();
        control.cancel().unwrap();
        let state: RunState = serde_json::from_slice(&fs::read(&control.path).unwrap()).unwrap();
        assert_eq!(state.status, "cancelled");
    }

    #[test]
    fn running_cancellation_waits_for_worker_outcome() {
        let root = tempfile::tempdir().unwrap();
        let control = queued_control(root.path());
        control.start().unwrap();
        control.cancel().unwrap();
        let state: RunState = serde_json::from_slice(&fs::read(&control.path).unwrap()).unwrap();
        assert_eq!(state.status, "running");
        // A completed execution remains successful even if cancellation arrives late.
        control.finish("succeeded", None).unwrap();
        let state: RunState = serde_json::from_slice(&fs::read(&control.path).unwrap()).unwrap();
        assert_eq!(state.status, "succeeded");
    }

    #[test]
    fn history_orders_timestamp_ties_and_legacy_states_without_losing_runs() {
        let root = tempfile::tempdir().unwrap();
        for (number, timestamp) in [(1, Some(5)), (2, Some(5)), (3, Some(4)), (4, None)] {
            let id = Uuid::from_u128(number).to_string();
            let path = root.path().join(&id);
            fs::create_dir(&path).unwrap();
            let mut state = serde_json::json!({"id":id,"status":"succeeded","error":null});
            if let Some(timestamp) = timestamp {
                state["created_at_ms"] = timestamp.into();
            }
            fs::write(path.join("state.json"), serde_json::to_vec(&state).unwrap()).unwrap();
        }
        let runs = Runs::new(root.path(), "test-history-token".into(), 1).unwrap();
        let mut before = None;
        let mut seen = Vec::new();
        loop {
            let page = history_page(&runs, 1, before).unwrap();
            seen.extend(page.runs.iter().map(|run| run.id.clone()));
            match page.next_cursor {
                None => break,
                Some(cursor) => {
                    let (time, id) = cursor.split_once('_').unwrap();
                    before = Some((time.parse().unwrap(), id.into()));
                }
            }
        }
        assert_eq!(
            seen,
            [2, 1, 3, 4].map(|number| Uuid::from_u128(number).to_string())
        );
    }

    #[test]
    fn indexed_history_reads_only_page_states_but_observes_current_status() {
        let root = tempfile::tempdir().unwrap();
        for number in 1..=100 {
            let id = Uuid::from_u128(number).to_string();
            let path = root.path().join(&id);
            fs::create_dir(&path).unwrap();
            let state = serde_json::json!({"id":id,"created_at_ms":number as u64,"status":"succeeded","error":null});
            fs::write(path.join("state.json"), serde_json::to_vec(&state).unwrap()).unwrap();
        }
        let store = Runs::new(root.path(), "test-history-token".into(), 1).unwrap();
        // An unrelated page's state must not be read on the newest-page path.
        let oldest = Uuid::from_u128(1).to_string();
        fs::write(
            root.path().join(&oldest).join("state.json"),
            b"invalid JSON",
        )
        .unwrap();
        let newest = Uuid::from_u128(100).to_string();
        let changed =
            serde_json::json!({"id":newest,"created_at_ms":100,"status":"failed","error":null});
        fs::write(
            root.path().join(&newest).join("state.json"),
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        let page = history_page(&store, 2, None).unwrap();
        assert_eq!(page.runs.len(), 2);
        assert_eq!(page.runs[0].id, newest);
        assert_eq!(page.runs[0].status, "failed");
        assert!(page.next_cursor.is_some());
        assert!(history_page(&store, 2, Some((2, Uuid::from_u128(2).to_string()))).is_err());
    }

    #[test]
    fn shutdown_closes_admission_and_joins_all_accepted_workers() {
        let root = tempfile::tempdir().unwrap();
        let runs = Runs::new(
            &root.path().join("store"),
            "test-lifecycle-token".into(),
            32,
        )
        .unwrap();
        let plan = plan(root.path());
        let first = admit(&runs, plan.clone(), opentelemetry::Context::new(), None)
            .unwrap_or_else(|(_, error)| panic!("{error}"));
        let barrier = Barrier::new(17);
        let accepted = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..16)
                .map(|_| {
                    let runs = &runs;
                    let plan = plan.clone();
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        admit(runs, plan, opentelemetry::Context::new(), None)
                    })
                })
                .collect();
            barrier.wait();
            runs.shutdown();
            handles
                .into_iter()
                .filter_map(|handle| match handle.join().unwrap() {
                    Ok(state) => Some(state.id),
                    Err((status, _)) => {
                        assert_eq!(status, 503);
                        None
                    }
                })
                .collect::<Vec<_>>()
        });
        assert!(runs.active.lock().unwrap().is_empty());
        for id in accepted.into_iter().chain([first.id]) {
            let state: RunState =
                serde_json::from_slice(&fs::read(runs.root.join(id).join("state.json")).unwrap())
                    .unwrap();
            assert_eq!(state.status, "cancelled");
        }
        // An already closed store must never restart work, including keyed requests.
        assert!(matches!(
            admit(&runs, plan, opentelemetry::Context::new(), Some("late")),
            Err((503, _))
        ));
        assert!(!submission_path(&runs, "late").exists());
        runs.shutdown();
    }
}
