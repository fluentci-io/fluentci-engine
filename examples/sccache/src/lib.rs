//! Compiler caching through a supervised native service and literal Cargo arguments.
use extism_pdk::*;
use fluentci_pdk::dag;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const INSTALL: &str = include_str!("install.sh");
const SERVER: &str = r#"set -eu
trap 'rm -f "$SCCACHE_SERVER_UDS"' EXIT
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; exit 0' TERM INT
SCCACHE_START_SERVER=1 SCCACHE_NO_DAEMON=1 sccache &
pid=$!
printf '%s\n' "$pid"
wait "$pid"
"#;

fn directory(name: &str, suffix: &str) -> Result<String, Error> {
    let configured = dag().get_env(name)?;
    let directory = if configured.is_empty() {
        let home = dag().get_env("HOME")?;
        if home.is_empty() {
            return Err(Error::msg(format!("set HOME or {name}")));
        }
        format!("{home}/{suffix}")
    } else {
        configured
    };
    if !directory.starts_with('/') {
        return Err(Error::msg(format!("{name} must be an absolute path")));
    }
    Ok(directory)
}

/// Input: [project_directory, fresh_run_directory, ...cargo_build_arguments].
/// Runs cargo build --release --locked and returns a native RunResult JSON report.
#[plugin_fn]
pub fn build(input: Json<Vec<String>>) -> FnResult<String> {
    let arguments = input.into_inner();
    if arguments.len() < 2 || arguments[0].is_empty() || arguments[1].is_empty() {
        return Err(Error::msg(
            "expected [project_directory, fresh_run_directory, ...cargo_build_arguments]",
        )
        .into());
    }
    if arguments[2..]
        .iter()
        .any(|arg| arg == "--profile" || arg.starts_with("--profile="))
    {
        return Err(Error::msg("this example always builds with the release profile").into());
    }
    let install = directory("SCCACHE_INSTALL_DIR", ".local/bin")?;
    let cache = directory("SCCACHE_DIR", ".cache/sccache")?;
    let path = format!("{install}:{}", dag().get_env("PATH")?);
    // A private socket avoids the runner's shared sccache daemon on TCP port 4226.
    let digest = format!("{:x}", Sha256::digest(arguments[1].as_bytes()));
    let socket = format!("/tmp/fluentci-sccache-{}.sock", &digest[..24]);
    let env = json!({
        "PATH": path, "SCCACHE_INSTALL_DIR": install, "SCCACHE_DIR": cache,
        "SCCACHE_CACHE_SIZE": "1G", "SCCACHE_SERVER_UDS": socket,
        "RUSTC_WRAPPER": "sccache", "CARGO_INCREMENTAL": "0"
    });
    let mut cargo = vec!["build", "--release", "--locked"];
    cargo.extend(arguments[2..].iter().map(String::as_str));
    let task = |id: &str, program: &str, args: Value, needs: Value| {
        json!({"id": id, "program": program, "args": args, "needs": needs,
               "cwd": arguments[0], "env": env})
    };
    let mut setup = task("setup", "/bin/bash", json!(["-c", INSTALL]), json!([]));
    setup["resources"] = json!([format!("sccache-install:{install}")]);
    setup["timeout_ms"] = json!(180_000);
    let mut server = task(
        "compiler-cache",
        "/bin/bash",
        json!(["-c", SERVER]),
        json!(["setup"]),
    );
    // Local sccache storage must have one server owner. Hold the lease until shutdown.
    server["resources"] = json!([format!("sccache-storage:{cache}")]);
    server["service"] = json!({
        "readiness": {"program": "/bin/sh", "args": ["-c", "test -S \"$SCCACHE_SERVER_UDS\""],
                      "interval_ms": 100, "timeout_ms": 1000},
        "startup_timeout_ms": 15_000, "shutdown_timeout_ms": 5000
    });
    let mut compile = task("build", "cargo", json!(cargo), json!(["compiler-cache"]));
    compile["timeout_ms"] = json!(1_800_000);
    let stats = task(
        "stats",
        "sccache",
        json!(["--show-stats", "--stats-format=json"]),
        json!(["build", "compiler-cache"]),
    );
    let plan = json!({"version": 1, "jobs": 1, "max_services": 1, "inherit_env": true,
                      "tasks": [setup, server, compile, stats]});
    let report = fluentci_pdk::run_plan(&plan.to_string(), &arguments[1])?;
    let report_value: Value = serde_json::from_str(&report)?;
    if report_value["outcome"] != "succeeded" {
        return Err(Error::msg(format!(
            "sccache build failed; see {}/result.json: {report}",
            arguments[1]
        ))
        .into());
    }
    Ok(report)
}
