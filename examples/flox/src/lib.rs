use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let cache_id = dag().cache("flox".into())?.id;
    let stdout = dag()
        .flox()?
        .with_workdir("./flox-demo".into())?
        .with_cache("./.flox".into(), cache_id)?
        .with_exec(command.split_whitespace().collect())?
        .stdout()?;
    Ok(stdout)
}
