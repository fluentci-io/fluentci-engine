use anyhow::Error;
use extism_pdk::*;
use fluentci_pdk::dag;

#[plugin_fn]
pub fn call(command: String) -> FnResult<String> {
    let command = command.split_whitespace().collect::<Vec<_>>();
    if command.len() < 2 {
        return Err(Error::msg("Invalid command").into());
    }
    let url = command[0].to_string();
    let function = command[1];
    let args = command[2..].to_vec();

    if !url.ends_with(".wasm") {
        return Err(Error::msg("Invalid module, must be a wasm file").into());
    }

    let response = dag().call(&url, &function, args)?;
    Ok(response)
}
