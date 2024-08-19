use extism::{convert::Json, *};
use fluentci_common::hermit::hermit as common_hermit;

use crate::state::State;

host_fn!(pub hermit(user_data: State;) -> Json<Hermit> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();

    let hermit = common_hermit(graph, true)?;
    Ok(Json(hermit))
});
