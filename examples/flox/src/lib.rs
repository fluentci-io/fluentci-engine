use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let cache_id = dag().cache("flox")?.id;
    let stdout = dag()
        .flox()?
        .with_workdir("./flox-demo")?
        .with_cache("./.flox", &cache_id)?
        .with_exec(command.split_whitespace().collect())?
        .stdout()?;
    Ok(stdout)
}
