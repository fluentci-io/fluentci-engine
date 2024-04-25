use extism_pdk::*;
use fluentci_pdk::dag;
use fluentci_types::nix::NixArgs;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let stdout = dag()
        .nix(NixArgs { impure: true })?
        .with_workdir("./nix-demo")?
        .with_exec(command.split_whitespace().collect())?
        .stdout()?;
    Ok(stdout)
}
