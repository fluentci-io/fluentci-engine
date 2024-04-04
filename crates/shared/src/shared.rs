use std::sync::Arc;

use extism::{convert::Json, host_fn};
use fluentci_common::common;
use fluentci_ext::devbox::Devbox as DevboxExt;
use fluentci_ext::devenv::Devenv as DevenvExt;
use fluentci_ext::envhub::Envhub as EnvhubExt;
use fluentci_ext::flox::Flox as FloxExt;
use fluentci_ext::git::Git as GitExt;
use fluentci_ext::http::Http as HttpExt;
use fluentci_ext::mise::Mise as MiseExt;
use fluentci_ext::nix::Nix as NixExt;
use fluentci_ext::pixi::Pixi as PixiExt;
use fluentci_ext::pkgx::Pkgx as PkgxExt;
use fluentci_ext::runner::Runner as RunnerExt;
use fluentci_ext::Extension;
use fluentci_types::cache::Cache;
use fluentci_types::file::File;

use crate::state::State;

host_fn!(pub set_runner(user_data: State; runner: String) {
  let state = user_data.get()?;
  let mut state = state.lock().unwrap();
  state.runner = runner;
  Ok(())
});

host_fn!(pub with_exec(user_data: State; args: Json<Vec<String>>) {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  let runner = state.runner.clone();
  let runner: Arc<Box<dyn Extension + Send + Sync>> = match runner.as_str() {
    "nix" => Arc::new(Box::new(NixExt::default())),
    "devbox" => Arc::new(Box::new(DevboxExt::default())),
    "devenv" => Arc::new(Box::new(DevenvExt::default())),
    "envhub" => Arc::new(Box::new(EnvhubExt::default())),
    "flox" => Arc::new(Box::new(FloxExt::default())),
    "git" => Arc::new(Box::new(GitExt::default())),
    "http" => Arc::new(Box::new(HttpExt::default())),
    "mise" => Arc::new(Box::new(MiseExt::default())),
    "runner" => Arc::new(Box::new(RunnerExt::default())),
    "pixi" => Arc::new(Box::new(PixiExt::default())),
    "pkgx" => Arc::new(Box::new(PkgxExt::default())),
    _ => Arc::new(Box::new(RunnerExt::default()))
  };
  common::with_exec(graph, args.into_inner(), runner);
  Ok(())
});

host_fn!(pub with_workdir(user_data: State; path: String) {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  let runner = state.runner.clone();
  let runner: Arc<Box<dyn Extension + Send + Sync>> = match runner.as_str() {
    "nix" => Arc::new(Box::new(NixExt::default())),
    "devbox" => Arc::new(Box::new(DevboxExt::default())),
    "devenv" => Arc::new(Box::new(DevenvExt::default())),
    "envhub" => Arc::new(Box::new(EnvhubExt::default())),
    "flox" => Arc::new(Box::new(FloxExt::default())),
    "git" => Arc::new(Box::new(GitExt::default())),
    "http" => Arc::new(Box::new(HttpExt::default())),
    "mise" => Arc::new(Box::new(MiseExt::default())),
    "runner" => Arc::new(Box::new(RunnerExt::default())),
    "pixi" => Arc::new(Box::new(PixiExt::default())),
    "pkgx" => Arc::new(Box::new(PkgxExt::default())),
    _ => Arc::new(Box::new(RunnerExt::default())),
  };
  common::with_workdir(graph, path, runner)?;
  Ok(())
});

host_fn!(pub with_cache(user_data: State;  cache: Json<Cache>) {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  let cache = cache.into_inner();
  common::with_cache(graph, cache.id, cache.path)?;
  Ok(())
});

host_fn!(pub with_file(user_data: State;  file: Json<File>) {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  let file = file.into_inner();
  common::with_file(graph, file.id, file.path)?;
  Ok(())
});

host_fn!(pub stdout(user_data: State;) -> String {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  let rx = state.rx.clone();
  common::stdout(graph, rx)
});

host_fn!(pub stderr(user_data: State;) -> String {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  let rx = state.rx.clone();
  common::stderr(graph, rx)
});

host_fn!(pub zip(user_data: State; path: String) -> Json<File> {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  match common::zip(graph, path) {
    Ok(file) => Ok(Json(File::from(file))),
    Err(e) => Err(e),
  }
});

host_fn!(pub tar_czvf(user_data: State; path: String) -> Json<File> {
  let state = user_data.get()?;
  let state = state.lock().unwrap();
  let graph = state.graph.clone();
  match common::tar_czvf(graph, path) {
    Ok(file) => Ok(Json(File::from(file))),
    Err(e) => Err(e),
  }
});

host_fn!(pub get_env(user_data: State; key: String) -> String {
  Ok(std::env::var(key).unwrap_or_default())
});

host_fn!(pub set_envs(user_data: State; env: Json<Vec<(String, String)>>) {
  for (key, value) in env.into_inner() {
    std::env::set_var(key, value);
  }
  Ok(())
});

host_fn!(pub remove_env(user_data: State; key: String) {
  std::env::remove_var(key);
  Ok(())
});

host_fn!(pub has_env(user_data: State; key: String) -> Json<bool> {
  Ok(Json(std::env::var(key).is_ok()))
});

host_fn!(pub get_os(user_data: State;) -> String {
  Ok(std::env::consts::OS.to_string())
});

host_fn!(pub get_arch(user_data: State;) -> String {
  Ok(std::env::consts::ARCH.to_string())
});
