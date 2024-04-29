use anyhow::Result;

pub mod aws;
pub mod azure;
pub mod convert;
pub mod google;
pub mod vault;

pub trait Vault {
    fn download_prefixed(&self, prefix: &str) -> Result<Vec<(String, String)>>;
    fn download_json(&self, secret_name: &str) -> Result<Vec<(String, String)>>;
}

pub trait VaultConfig {
    type Vault: Vault;
    fn is_enabled(&self) -> bool;
    fn into_vault(self) -> Result<Self::Vault>;
}

pub fn download_env() {}
