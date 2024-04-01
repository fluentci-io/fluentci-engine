use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let stdout = dag()
        .mise()?
        .with_workdir("./mise-demo")?
        .with_exec(command.split_whitespace().collect())?
        .stdout()?;
    Ok(stdout)
}
