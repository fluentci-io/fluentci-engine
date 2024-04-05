use std::sync::{mpsc, Arc, Mutex};

use extism::{Manifest, PluginBuilder, UserData, Wasm, PTR};
use fluentci_core::deps::Graph;
use fluentci_ext::runner::Runner;
use fluentci_shared::{
    cache::*,
    devbox::*,
    devenv::*,
    directory::*,
    envhub::*,
    file::*,
    flox::*,
    git::*,
    http::*,
    mise::*,
    nix::*,
    pipeline::*,
    pixi::*,
    pkgx::*,
    shared::{self, *},
    state::State,
};

pub fn call(module: &str, command: &str) {
    let (tx, rx) = mpsc::channel();
    let user_data = UserData::new(State {
        graph: Arc::new(Mutex::new(Graph::new(
            tx,
            Arc::new(Box::new(Runner::default())),
        ))),
        rx: Arc::new(Mutex::new(rx)),
        runner: "default".into(),
    });

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
        .with_function("call", [PTR], [PTR], user_data.clone(), shared::call)
        .build()
        .unwrap();

    let func = command.split_whitespace().next().unwrap();
    let args = command.split_whitespace().skip(1).collect::<Vec<&str>>();
    let args = args.join(" ");
    match plugin.call::<&str, &str>(func, &args) {
        Ok(res) => println!("{}", res),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    }
}
