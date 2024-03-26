use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Devbox {
    pub id: String,
}

host_fn!(pub devbox(user_data: Graph;) -> Json<Devbox> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Devbox {
        id,
    }))
});
