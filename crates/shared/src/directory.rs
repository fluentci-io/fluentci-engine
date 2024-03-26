use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Directory {
    pub id: String,
    pub path: String,
}

host_fn!(pub directory(user_data: Graph; path: String) -> Json<Directory> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Directory {
        id,
        path,
    }))
});
