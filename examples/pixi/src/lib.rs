use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let stdout = dag()
        .pixi()?
        .with_workdir("./pixi-demo")?
        .with_exec(command.split_whitespace().collect())?
        .stdout()?;
    Ok(stdout)
}
