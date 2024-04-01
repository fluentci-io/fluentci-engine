use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn git_tree(url: String) -> FnResult<Json<Vec<String>>> {
    let entries = dag().git(&url)?.branch("main")?.tree()?.entries()?;
    Ok(Json(entries))
}
