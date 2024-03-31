use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let stdout = dag()
        .nix()?
        .with_workdir("./nix-demo".into())?
        .with_exec(command.split_whitespace().map(|s| s.to_string()).collect())?
        .stdout()?;
    Ok(stdout)
}