use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Cache {
    pub id: String,
    pub key: String,
}

host_fn!(pub cache(user_data: Graph; key: String) -> Json<Cache> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Cache {
        id,
        key,
    }))
});
