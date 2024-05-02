use anyhow::Error;
use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn get_secret(name: String) -> FnResult<String> {
    let address = dag().get_env("VAULT_ADDR")?;
    let token = dag().get_env("VAULT_TOKEN")?;
    let secrets = dag()
        .hashicorp_vault(&address, &token, None)?
        .get_secret(&name)?;

    if secrets.is_empty() {
        return Err(WithReturnCode::from(Error::msg("secret not found")).into());
    }

    let stdout = dag()
        .pipeline("demo")?
        .with_secret_variable("DEMO", &secrets[0].id)?
        .with_exec(vec!["echo", "$DEMO"])?
        .stdout()?;

    Ok(stdout)
}
