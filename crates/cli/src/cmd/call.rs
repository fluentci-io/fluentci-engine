use std::sync::{mpsc, Arc};

use extism::{Manifest, PluginBuilder, UserData, Wasm, PTR};
use fluentci_core::deps::Graph;
use fluentci_ext::runner::Runner;
use fluentci_shared::{
    cache::*, devbox::*, devenv::*, directory::*, envhub::*, file::*, flox::*, git::*, http::*,
    nix::*, pipeline::*, pixi::*, pkgx::*,
};

pub fn call() {
    let (tx, _rx) = mpsc::channel();
    let user_data = UserData::new(Graph::new(tx, Arc::new(Box::new(Runner::default()))));
    let url =
        Wasm::url("https://github.com/extism/plugins/releases/latest/download/count_vowels.wasm");
    let manifest = Manifest::new([url]);
    let mut plugin = PluginBuilder::new(manifest)
        .with_wasi(true)
        .with_function("cache", [PTR], [PTR], user_data.clone(), cache)
        .with_function("devbox", [PTR], [PTR], user_data.clone(), devbox)
        .with_function("devenv", [PTR], [PTR], user_data.clone(), devenv)
        .with_function("directory", [PTR], [PTR], user_data.clone(), directory)
        .with_function("envhub", [PTR], [PTR], user_data.clone(), envhub)
        .with_function("file", [PTR], [PTR], user_data.clone(), file)
        .with_function("flox", [PTR], [PTR], user_data.clone(), flox)
        .with_function("git", [PTR], [PTR], user_data.clone(), git)
        .with_function("http", [PTR], [PTR], user_data.clone(), http)
        .with_function("nix", [PTR], [PTR], user_data.clone(), nix)
        .with_function("pipeline", [PTR], [PTR], user_data.clone(), pipeline)
        .with_function("pixi", [PTR], [PTR], user_data.clone(), pixi)
        .with_function("pkgx", [PTR], [PTR], user_data.clone(), pkgx)
        .build()
        .unwrap();

    for _ in 0..5 {
        let res = plugin
            .call::<&str, &str>("count_vowels", "Hello, world!")
            .unwrap();
        println!("{}", res);
    }
}
