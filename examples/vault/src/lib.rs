use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn get_secret(name: String) -> FnResult<String> {
    let address = dag().get_env("VAULT_ADDR")?;
    let token = dag().get_env("VAULT_TOKEN")?;
    let secrets = dag()
        .hashicorp_vault(&address, &token, None)?
        .get_secret(&name)?;
    Ok(secrets
        .into_iter()
        .map(|s| s.plaintext().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n"))
}
