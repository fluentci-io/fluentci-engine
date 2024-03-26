use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Flox {
    pub id: String,
}

host_fn!(pub flox(user_data: Graph;) -> Json<Flox> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Flox {
        id,
    }))
});
