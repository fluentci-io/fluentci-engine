use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct File {
    pub id: String,
}

host_fn!(pub file(user_data: Graph;) -> Json<File> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(File {
        id,
    }))
});
