use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn get_env(key: String) -> FnResult<String> {
    let value = dag().get_env(&key)?;
    Ok(value)
}

#[plugin_fn]
pub fn has_env(key: String) -> FnResult<Json<bool>> {
    let value = dag().has_env(&key)?;
    Ok(Json(value))
}

#[plugin_fn]
pub fn set_envs(env: Json<Vec<(String, String)>>) -> FnResult<()> {
    let env = env.into_inner();
    dag().set_envs(env)?;
    Ok(())
}

#[plugin_fn]
pub fn remove_env(key: String) -> FnResult<()> {
    dag().remove_env(&key)?;
    Ok(())
}

#[plugin_fn]
pub fn get_os() -> FnResult<String> {
    let os = dag().get_os()?;
    Ok(os)
}

#[plugin_fn]
pub fn get_arch() -> FnResult<String> {
    let arch = dag().get_arch()?;
    Ok(arch)
}
