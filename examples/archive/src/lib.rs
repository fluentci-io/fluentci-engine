use extism_pdk::*;
use fluentci_pdk::{dag, file::File};

#[plugin_fn]
pub fn tar_czvf(path: String) -> FnResult<Json<File>> {
    let file = dag().directory(&path)?.tar_czvf()?;
    Ok(Json(file))
}

#[plugin_fn]
pub fn zip(path: String) -> FnResult<Json<File>> {
    let file = dag().directory(&path)?.zip()?;
    Ok(Json(file))
}

#[plugin_fn]
pub fn tar_xzvf(path: String) -> FnResult<Json<Vec<String>>> {
    let entries = dag().file(&path)?.tar_xzvf()?.entries()?;
    Ok(Json(entries))
}

#[plugin_fn]
pub fn unzip(path: String) -> FnResult<Json<Vec<String>>> {
    let entries = dag().file(&path)?.unzip()?.entries()?;
    Ok(Json(entries))
}
