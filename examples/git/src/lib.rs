use extism_pdk::*;
use fluentci_client::dag;

#[plugin_fn]
pub fn git_tree(url: String) -> FnResult<String> {
    let entries = dag()
        .git(url)?
        .branch("main".into())?
        .tree()?
        .with_exec(
            vec!["ls", "-l"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )?
        .stdout()?;
    Ok(entries)
}
