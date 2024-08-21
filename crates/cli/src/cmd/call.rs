use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
    fs,
    process::{Command, Stdio},
    sync::{mpsc, Arc, Mutex},
};

use anyhow::Error;
use extism::{Manifest, PluginBuilder, UserData, Wasm, PTR};
use fluentci_core::deps::Graph;
use fluentci_ext::{pkgx::Pkgx, runner::Runner, Extension};
use fluentci_shared::{
    cache::*,
    devbox::*,
    devenv::*,
    directory::*,
    envhub::*,
    file::*,
    flox::*,
    git::*,
    hermit::*,
    http::*,
    mise::*,
    nix::*,
    pipeline::*,
    pixi::*,
    pkgx::*,
    proto::*,
    shared::{self, *},
    state::State,
};

pub fn call(module: &str, command: &str) -> Result<(), Error> {
    match fluentci_core::init_tracer() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e.to_string());
        }
    }
    match fluentci_core::set_git_repo_metadata() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e.to_string());
        }
    }

    let (tx, rx) = mpsc::channel();
    let user_data = UserData::new(State {
        graph: Arc::new(Mutex::new(Graph::new(
            tx,
            Arc::new(Box::new(Runner::default())),
        ))),
        rx: Arc::new(Mutex::new(rx)),
        runner: "default".into(),
    });

    let module = match module.starts_with("http")
        || module.starts_with("azurecr.io/")
        || module.starts_with("ghcr.io/")
        || module.starts_with("gcr.io/")
    {
        true => download_module(module)?,
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
        .with_function("nix", [PTR], [PTR], user_data.clone(), nix)
        .with_function("pipeline", [PTR], [PTR], user_data.clone(), pipeline)
        .with_function("pixi", [], [PTR], user_data.clone(), pixi)
        .with_function("pkgx", [], [PTR], user_data.clone(), pkgx)
        .with_function("proto", [], [PTR], user_data.clone(), proto)
        .with_function("mise", [], [PTR], user_data.clone(), mise)
        .with_function("trust", [], [], user_data.clone(), trust)
        .with_function("hermit", [], [PTR], user_data.clone(), hermit)
        .with_function("install", [], [], user_data.clone(), install)
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
        .with_function("with_packages", [PTR], [], user_data.clone(), with_packages)
        .with_function("as_service", [PTR], [PTR], user_data.clone(), as_service)
        .with_function("with_service", [PTR], [], user_data.clone(), with_service)
        .with_function("wait_on", [PTR], [], user_data.clone(), wait_on)
        .with_function(
            "add_secretmanager",
            [PTR],
            [PTR],
            user_data.clone(),
            add_secretmanager,
        )
        .with_function("get_secret", [PTR], [PTR], user_data.clone(), get_secret)
        .with_function("set_secret", [PTR], [PTR], user_data.clone(), set_secret)
        .with_function(
            "with_secret_variable",
            [PTR],
            [],
            user_data.clone(),
            with_secret_variable,
        )
        .with_function(
            "get_secret_plaintext",
            [PTR],
            [PTR],
            user_data.clone(),
            get_secret_plaintext,
        )
        .build()
        .unwrap();

    let func = command.split_whitespace().next().unwrap();
    let args = command.split_whitespace().skip(1).collect::<Vec<&str>>();
    let args = args.join(" ");
    match plugin.call::<&str, &str>(func, &args) {
        Ok(res) => {
            println!("{}", res);
        }
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
    Ok(())
}

pub fn download_module(url: &str) -> Result<Wasm, Error> {
    let filename = format!("{}.wasm", sha256::digest(url).to_string());
    let work_dir = format!("{}/.fluentci/cache", std::env::var("HOME").unwrap());

    if fs::metadata(format!("{}/{}", work_dir, filename)).is_ok() {
        return Ok(Wasm::file(format!("{}/{}", work_dir, filename)));
    }

    Pkgx::default().setup()?;

    if url.starts_with("azurecr.io/") || url.starts_with("ghcr.io/") || url.starts_with("gcr.io/") {
        fs::create_dir_all(&work_dir)?;
        setup_wasm_to_oci()?;

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(format!("wasm-to-oci pull {} --out {}", url, filename))
            .current_dir(&work_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        child.wait()?;

        return Ok(Wasm::file(format!("{}/{}", work_dir, filename)));
    }

    let cmd = format!(
        "pkgx +rockdaboot.github.io/libpsl +curl.se curl -s {} -o {}",
        url, filename
    );
    fs::create_dir_all(&work_dir)?;

    let mut child = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .current_dir(&work_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    child.wait()?;

    Ok(Wasm::file(format!("{}/{}", work_dir, filename)))
}

pub fn setup_wasm_to_oci() -> Result<(), Error> {
    let os = match OS {
        "macos" => "darwin",
        _ => OS,
    };
    let arch = match ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => ARCH,
    };

    std::env::set_var("OS", os);
    std::env::set_var("ARCH", arch);

    let path = std::env::var("PATH").unwrap();
    let home = std::env::var("HOME").unwrap();
    let path = format!("{}/.local/bin:{}", home, path);
    std::env::set_var("PATH", path);

    fs::create_dir_all(format!("{}/.local/bin", home))?;

    let mut child = Command::new("bash")
        .arg("-c")
        .arg("type wasm-to-oci > /dev/null 2> /dev/null || pkgx wget https://github.com/fluentci-io/wasm-to-oci/releases/download/v0.1.2/wasm-to-oci_${OS}-${ARCH}.tar.gz ; \
            type wasm-to-oci > /dev/null 2> /dev/null || pkgx tar xvf wasm-to-oci_${OS}-${ARCH}.tar.gz ; \
            type wasm-to-oci > /dev/null 2> /dev/null || cp wasm-to-oci ${HOME}/.local/bin ; \
            [ -f wasm-to-oci ] && rm wasm-to-oci || true ; \
            [ -f wasm-to-oci_${OS}-${ARCH}.tar.gz ] && rm wasm-to-oci_${OS}-${ARCH}.tar.gz || true \
        ")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    child.wait()?;

    Ok(())
}
