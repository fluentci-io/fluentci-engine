use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Http {
    pub id: String,
}

host_fn!(pub http(user_data: Graph;) -> Json<Http> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Http {
        id,
    }))
});
