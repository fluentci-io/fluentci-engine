use std::{env, thread};

use anyhow::Error;
use baselime::Level;

pub mod baselime;

pub fn error(data: &str, namespace: &str) -> Result<(), Error> {
    if let Ok(_) = env::var("BASELIME_API_KEY") {
        let data = data.to_string();
        let namespace = namespace.to_string();
        thread::spawn(move || {
            let baselime_client = baselime::new();
            match baselime_client.send(data, Level::Error, namespace) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        });
    }
    Ok(())
}

pub fn info(data: &str, namespace: &str) -> Result<(), Error> {
    if let Ok(_) = env::var("BASELIME_API_KEY") {
        let data = data.to_string();
        let namespace = namespace.to_string();
        thread::spawn(move || {
            let baselime_client = baselime::new();
            match baselime_client.send(data, Level::Info, namespace) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        });
    }
    Ok(())
}
