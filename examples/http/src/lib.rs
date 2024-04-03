use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn get(url: String) -> FnResult<String> {
    let file = dag().http(&url)?;
    let stdout = dag()
        .pipeline("http-demo")?
        .with_file("./demo.html", &file.id)?
        .with_exec(vec!["ls", "-l"])?
        .stdout()?;
    Ok(stdout)
}
