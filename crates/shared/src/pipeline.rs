use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
}

host_fn!(pub pipeline(user_data: Graph;) -> Json<Pipeline> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Pipeline {
        id,
    }))
});
