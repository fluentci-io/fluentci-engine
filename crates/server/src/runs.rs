//! Independent, authenticated run API. No mutable current-pipeline state.
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use fluentci_executor::{Cancellation, Plan};
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    thread::JoinHandle,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct RunState {
    id: String,
    status: String,
    error: Option<String>,
}
struct Active {
    cancel: Cancellation,
    worker: JoinHandle<()>,
}
pub struct Runs {
    _store_lock: fs::File,
    root: PathBuf,
    token: String,
    max_runs: usize,
    active: Mutex<HashMap<String, Active>>,
}
fn save(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let mut file =
        tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, value).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
impl Runs {
    pub fn new(root: &Path, token: String, max_runs: usize) -> Result<Self, String> {
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
        // A previous process cannot own live runs in this store. Never silently
        // report them as running after a restart.
        for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if Uuid::parse_str(&entry.file_name().to_string_lossy()).is_err() {
                continue;
            }
            let path = entry.path().join("state.json");
            if let Ok(bytes) = fs::read(&path) {
                let mut state: RunState =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if state.status == "running" {
                    if let Ok(result) = fs::read(entry.path().join("execution/result.json")) {
                        let result: serde_json::Value =
                            serde_json::from_slice(&result).map_err(|e| e.to_string())?;
                        let success = result["tasks"]
                            .as_array()
                            .is_some_and(|tasks| tasks.iter().all(|t| t["status"] == "succeeded"));
                        state.status = if success { "succeeded" } else { "failed" }.into();
                    } else {
                        state.status = "interrupted".into();
                        state.error = Some("server stopped before the run completed".into());
                    }
                    save(&path, &state)?;
                }
            }
        }
        Ok(Self {
            _store_lock: store_lock,
            root,
            token,
            max_runs,
            active: Mutex::new(HashMap::new()),
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
        let active = std::mem::take(&mut *self.active.lock().unwrap());
        for run in active.values() {
            run.cancel.cancel();
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

async fn submit(
    request: HttpRequest,
    runs: web::Data<Runs>,
    plan: web::Json<Plan>,
) -> HttpResponse {
    if !runs.authorized(&request) {
        return HttpResponse::Unauthorized().finish();
    }
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
    match tokio::task::spawn_blocking(move || admit(&runs, plan, parent)).await {
        Ok(Ok(state)) => HttpResponse::Accepted()
            .insert_header(("Location", format!("/v1/runs/{}", state.id)))
            .json(state),
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
) -> Result<RunState, (u16, String)> {
    if plan.jobs > 64 || plan.max_services > 32 || plan.tasks.len() > 10_000 {
        return Err((
            400,
            "plan exceeds server limits (64 jobs, 32 services, 10000 tasks)".into(),
        ));
    }
    if let Err(error) = plan.validate() {
        return Err((400, error));
    }
    let mut active = runs.active.lock().unwrap();
    active.retain(|_, run| !run.worker.is_finished());
    if active.len() >= runs.max_runs {
        return Err((429, "run capacity reached".into()));
    }
    let id = Uuid::new_v4().to_string();
    let path = runs.root.join(&id);
    let state = RunState {
        id: id.clone(),
        status: "running".into(),
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
        save(&path.join("state.json"), &state)
    })();
    if preparation.is_err() {
        return Err((500, "could not persist run".into()));
    }
    let cancel = Cancellation::default();
    let worker_cancel = cancel.clone();
    let worker_path = path.clone();
    let worker_id = id.clone();
    let worker = std::thread::Builder::new()
        .name(format!("run-{id}"))
        .spawn(move || {
            let _guard = parent.attach();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                plan.execute(&worker_path.join("execution"), &worker_cancel)
            }));
            let (status, error) = match outcome {
                Ok(Ok(result)) if result.success() => ("succeeded", None),
                Ok(Ok(_)) if worker_cancel.is_cancelled() => ("cancelled", None),
                Ok(Ok(_)) => ("failed", None),
                Ok(Err(error)) => ("failed", Some(error)),
                Err(_) => ("failed", Some("executor panicked".into())),
            };
            if let Err(error) = save(
                &worker_path.join("state.json"),
                &RunState {
                    id: worker_id,
                    status: status.into(),
                    error,
                },
            ) {
                eprintln!("could not persist run completion: {error}");
            }
        });
    match worker {
        Ok(worker) => {
            active.insert(id.clone(), Active { cancel, worker });
            Ok(state)
        }
        Err(_) => {
            let _ = save(
                &path.join("state.json"),
                &RunState {
                    id,
                    status: "failed".into(),
                    error: Some("could not start run worker".into()),
                },
            );
            Err((500, "could not start run worker".into()))
        }
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
    if let Some(run) = runs.active.lock().unwrap().get(id.as_str()) {
        run.cancel.cancel();
    }
    HttpResponse::Accepted().finish()
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
    if index >= 10_000 || !matches!(stream.as_str(), "stdout" | "stderr") {
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
        .route("/v1/runs/{id}", web::get().to(state))
        .route("/v1/runs/{id}/result", web::get().to(result))
        .route("/v1/runs/{id}/cancel", web::post().to(cancel))
        .route("/v1/runs/{id}/events", web::get().to(events))
        .route("/v1/runs/{id}/logs/{index}/{stream}", web::get().to(logs));
}
pub async fn start(listen: &str, root: &Path, max_runs: usize) -> anyhow::Result<()> {
    let token = std::env::var("FLUENTCI_RUN_TOKEN")
        .map_err(|_| anyhow::anyhow!("set FLUENTCI_RUN_TOKEN to authenticate run requests"))?;
    let runs = web::Data::new(Runs::new(root, token, max_runs).map_err(anyhow::Error::msg)?);
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
