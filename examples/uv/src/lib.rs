//! A small composable Python pipeline: setup uv, then run a project.
use extism_pdk::*;
use fluentci_pdk::{dag, pipeline::Pipeline};

const INSTALL: &str = r#"set -eu
if ! command -v uv >/dev/null 2>&1; then
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  /usr/bin/curl --fail --silent --show-error --location --retry 3 \
    https://astral.sh/uv/0.13.7/install.sh --output "$installer"
  PATH=/usr/bin:/bin:/usr/sbin:/sbin /bin/sh "$installer"
fi
uv --version"#;

fn setup_pipeline() -> Result<Pipeline, Error> {
    let install = match dag().get_env("UV_UNMANAGED_INSTALL")? {
        value if !value.is_empty() => value,
        _ => {
            let home = dag().get_env("HOME")?;
            if home.is_empty() {
                return Err(Error::msg("set HOME or UV_UNMANAGED_INSTALL for uv setup"));
            }
            format!("{home}/.local/bin")
        }
    };
    let path = dag().get_env("PATH")?;
    dag()
        .pipeline("uv")?
        .with_env_variable("UV_UNMANAGED_INSTALL", &install)?
        .with_env_variable("PATH", &format!("{install}:{path}"))?
        .with_exec(vec![INSTALL])
}
fn quote(argument: &str) -> String {
    format!("'{}'", argument.replace('\'', "'\\''"))
}

/// Reuse uv on PATH, or install the pinned binary without changing shell profiles.
#[plugin_fn]
pub fn setup() -> FnResult<String> {
    Ok(setup_pipeline()?.stdout()?)
}

/// Input: [project_directory, script, ...script_arguments]. Boundaries are preserved.
#[plugin_fn]
pub fn run(arguments: Json<Vec<String>>) -> FnResult<String> {
    let arguments = arguments.into_inner();
    if arguments.len() < 2 {
        return Err(Error::msg("uv run expects [project_directory, script, ...arguments]").into());
    }
    let mut command = vec!["uv", "run", "--project", ".", "python"];
    command.extend(arguments[1..].iter().map(String::as_str));
    let command = command.into_iter().map(quote).collect::<Vec<_>>().join(" ");
    Ok(setup_pipeline()?
        .with_workdir(&arguments[0])?
        .with_exec(vec![&command])?
        .stdout()?)
}
