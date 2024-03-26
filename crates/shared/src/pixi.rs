use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Pixi {
    pub id: String,
}

host_fn!(pub pixi(user_data: Graph;) -> Json<Pixi> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Pixi {
        id,
    }))
});
