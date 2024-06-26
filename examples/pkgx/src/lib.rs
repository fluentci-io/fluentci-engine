use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    let stdout = dag()
        .pkgx()?
        .with_workdir("./pkgx-demo")?
        .with_exec(command.split_whitespace().collect())?
        .stdout()?;
    Ok(stdout)
}
