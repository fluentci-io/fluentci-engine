use extism_pdk::{host_fn, Error, Json};
use fluentci_types::secret as types;
use serde::{Deserialize, Serialize};

use crate::secret::Secret;

#[host_fn]
extern "ExtismHost" {
    fn get_secret(params: Json<Vec<String>>) -> Json<Vec<types::Secret>>;
}

#[derive(Serialize, Deserialize)]
pub struct SecretManager {
    pub id: String,
}

impl SecretManager {
    pub fn get_secret(&self, name: &str) -> Result<Vec<Secret>, Error> {
        let results = unsafe { get_secret(Json(vec![self.id.clone(), name.into()]))? };
        Ok(results.into_inner().into_iter().map(Into::into).collect())
    }
}
