use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn md5(file: String) -> FnResult<String> {
    let hash = dag().file(&file)?.md5()?;
    Ok(hash)
}

#[plugin_fn]
pub fn sha256(file: String) -> FnResult<String> {
    let hash = dag().file(&file)?.sha256()?;
    Ok(hash)
}
