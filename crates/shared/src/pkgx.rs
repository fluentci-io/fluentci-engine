use extism::{convert::Json, *};
use fluentci_core::deps::Graph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Pkgx {
    pub id: String,
}

host_fn!(pub pkgx(user_data: Graph;) -> Json<Pkgx> {
  let id = Uuid::new_v4().to_string();
    Ok(Json(Pkgx {
        id,
    }))
});
