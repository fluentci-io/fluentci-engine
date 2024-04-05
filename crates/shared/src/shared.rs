use std::sync::Arc;

use extism::{convert::Json, host_fn};
use extism::{Manifest, PluginBuilder, Wasm, PTR};
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
use fluentci_types::Module;

use crate::{
    cache::*, devbox::*, devenv::*, directory::*, envhub::*, file::*, flox::*, git::*, http::*,
    mise::*, nix::*, pipeline::*, pixi::*, pkgx::*, state::State,
};

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

host_fn!(pub call(user_data: State; opts: Json<Module>) -> String {
    let opts = opts.into_inner();
    let module = opts.url.clone();
    let module = match module.starts_with("http") {
        true => Wasm::url(module),
        false => Wasm::file(module),
    };

    let manifest = Manifest::new([module]);
    let mut plugin = PluginBuilder::new(manifest.clone())
        .with_wasi(true)
        .with_function("set_runner", [PTR], [], user_data.clone(), set_runner)
        .with_function("cache", [PTR], [PTR], user_data.clone(), cache)
        .with_function("devbox", [], [PTR], user_data.clone(), devbox)
        .with_function("devenv", [], [PTR], user_data.clone(), devenv)
        .with_function("directory", [PTR], [PTR], user_data.clone(), directory)
        .with_function("entries", [PTR], [PTR], user_data.clone(), entries)
        .with_function("envhub", [], [PTR], user_data.clone(), envhub)
        .with_function("unzip", [PTR], [PTR], user_data.clone(), unzip)
        .with_function("file", [PTR], [PTR], user_data.clone(), file)
        .with_function("flox", [], [PTR], user_data.clone(), flox)
        .with_function("git", [PTR], [PTR], user_data.clone(), git)
        .with_function("branch", [PTR], [], user_data.clone(), branch)
        .with_function("commit", [], [PTR], user_data.clone(), commit)
        .with_function("tree", [], [PTR], user_data.clone(), tree)
        .with_function("http", [PTR], [PTR], user_data.clone(), http)
        .with_function("nix", [], [PTR], user_data.clone(), nix)
        .with_function("pipeline", [PTR], [PTR], user_data.clone(), pipeline)
        .with_function("pixi", [], [PTR], user_data.clone(), pixi)
        .with_function("pkgx", [], [PTR], user_data.clone(), pkgx)
        .with_function("mise", [], [PTR], user_data.clone(), mise)
        .with_function("with_exec", [PTR], [], user_data.clone(), with_exec)
        .with_function("with_workdir", [PTR], [], user_data.clone(), with_workdir)
        .with_function("with_cache", [PTR], [], user_data.clone(), with_cache)
        .with_function("stdout", [], [PTR], user_data.clone(), stdout)
        .with_function("stderr", [], [PTR], user_data.clone(), stderr)
        .with_function("zip", [PTR], [PTR], user_data.clone(), zip)
        .with_function("tar_czvf", [PTR], [PTR], user_data.clone(), tar_czvf)
        .with_function("tar_xzvf", [PTR], [PTR], user_data.clone(), tar_xzvf)
        .with_function("md5", [PTR], [PTR], user_data.clone(), md5)
        .with_function("sha256", [PTR], [PTR], user_data.clone(), sha256)
        .with_function("chmod", [PTR], [PTR], user_data.clone(), chmod)
        .with_function("with_file", [PTR], [], user_data.clone(), with_file)
        .with_function("get_env", [PTR], [PTR], user_data.clone(), get_env)
        .with_function("set_envs", [PTR], [], user_data.clone(), set_envs)
        .with_function("remove_env", [PTR], [], user_data.clone(), remove_env)
        .with_function("has_env", [PTR], [PTR], user_data.clone(), has_env)
        .with_function("get_os", [], [PTR], user_data.clone(), get_os)
        .with_function("get_arch", [], [PTR], user_data.clone(), get_arch)
        .with_function("call", [PTR], [PTR], user_data.clone(), call)
        .build()
        .unwrap();

      let func = opts.function.clone();
      let args = opts.args.clone();
      let result = plugin.call::<&str, &str>(func, &args)?;
      Ok(result.to_string())
});
