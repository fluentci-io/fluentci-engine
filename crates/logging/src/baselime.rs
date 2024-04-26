use std::env;

use anyhow::Error;
use reqwest::blocking::Client;
use serde::Serialize;
use uuid::Uuid;

const URL: &str = "https://events.baselime.io/v1/logs";

pub enum Level {
    Info,
    Error,
}

#[derive(Serialize, Default)]
pub struct Payload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub namespace: String,
}

pub struct Baselime {
    client: Client,
}

pub fn new() -> Baselime {
    let client = reqwest::blocking::Client::new();
    Baselime { client }
}

impl Baselime {
    pub fn send(&self, data: &str, level: Level, namespace: String) -> Result<(), Error> {
        match env::var("BASELIME_API_KEY") {
            Ok(api_key) => {
                let json = match level {
                    Level::Info => serde_json::to_string(&vec![Payload {
                        message: Some(data.to_string()),
                        request_id: Uuid::new_v4().to_string(),
                        namespace,
                        ..Default::default()
                    }])?,
                    Level::Error => serde_json::to_string(&vec![Payload {
                        error: Some(data.to_string()),
                        request_id: Uuid::new_v4().to_string(),
                        namespace,
                        ..Default::default()
                    }])?,
                };
                self.client
                    .post(URL)
                    .header("x-api-key", api_key)
                    .header("x-service", "fluentci-core")
                    .header("Content-Type", "application/json")
                    .body(json)
                    .send()?;
                Ok(())
            }
            Err(e) => Err(Error::msg(format!("BASELIME_API_KEY not set: {}", e)))?,
        }
    }
}
