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

// Migration regression fixtures: no toolchain providers or installers required.
#[plugin_fn]
pub fn exec(command: String) -> FnResult<String> {
    dag()
        .pipeline("legacy-exec")?
        .with_exec(vec![&command])?
        .stdout()
        .map_err(Into::into)
}

#[plugin_fn]
pub fn service_smoke(directory: String) -> FnResult<String> {
    let service = dag()
        .pipeline("legacy-service")?
        .with_workdir(&directory)?
        .with_exec(vec!["echo $$ > service.pid; while :; do sleep 1; done"])?
        .as_service("legacy-db")?;
    dag()
        .pipeline("legacy-client")?
        .with_workdir(&directory)?
        .with_service(&service)?
        .with_exec(vec!["test -s service.pid && printf healthy"])?
        .stdout()
        .map_err(Into::into)
}
