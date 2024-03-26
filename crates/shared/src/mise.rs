use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Mise {
    pub id: String,
}

host_fn!(pub mise(user_data: Graph;) -> Json<Mise> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Mise {
        id,
    }))
});
