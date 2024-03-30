use extism_pdk::*;
use fluentci_client::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let stdout = dag()
        .mise()?
        .with_workdir("./mise-demo".into())?
        .with_exec(command.split_whitespace().map(|s| s.to_string()).collect())?
        .stdout()?;
    Ok(stdout)
}
