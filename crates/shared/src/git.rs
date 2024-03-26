use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Git {
    pub id: String,
}

host_fn!(pub git(user_data: Graph;) -> Json<Git> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Git {
        id,
    }))
});
