use extism_pdk::*;
use fluentci_pdk::{dag, file::File};

#[plugin_fn]
pub fn chmod(opts: String) -> FnResult<Json<File>> {
    let opts = opts.split(" ").collect::<Vec<&str>>();
    let mode = opts[0];
    let path = opts[1];
    let file = dag().file(path)?.chmod(mode)?;
    Ok(Json(file))
}
