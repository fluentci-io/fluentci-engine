use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Devenv {
    pub id: String,
}

host_fn!(pub devenv(user_data: Graph;) -> Json<Devenv> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Devenv {
        id,
    }))
});
