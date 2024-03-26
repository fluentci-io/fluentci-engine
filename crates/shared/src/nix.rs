use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Nix {
    pub id: String,
}

host_fn!(pub nix(user_data: Graph;) -> Json<Nix> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Nix {
        id,
    }))
});
