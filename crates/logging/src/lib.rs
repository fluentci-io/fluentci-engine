use std::env;

use anyhow::Error;
use baselime::Level;

pub mod baselime;

pub fn error(data: &str, namespace: &str) -> Result<(), Error> {
    if let Ok(_) = env::var("BASELIME_API_KEY") {
        let baselime_client = baselime::new();
        baselime_client.send(data, Level::Error, namespace.to_string())?;
    }
    Ok(())
}

pub fn info(data: &str, namespace: &str) -> Result<(), Error> {
    if let Ok(_) = env::var("BASELIME_API_KEY") {
        let baselime_client = baselime::new();
        baselime_client.send(data, Level::Info, namespace.to_string())?;
    }
    Ok(())
}
