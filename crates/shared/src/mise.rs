use extism::{convert::Json, *};
use fluentci_common::mise::mise as common_mise;

use crate::state::State;

host_fn!(pub mise(user_data: State;) -> Json<Mise> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    let mise = common_mise(graph, true)?;
    Ok(Json(mise))
});
