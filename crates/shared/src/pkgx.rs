use extism::{convert::Json, *};
use fluentci_common::pkgx::pkgx as common_pkgx;

use crate::state::State;

host_fn!(pub pkgx(user_data: State;) -> Json<Pkgx> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    let pkgx = common_pkgx(graph, true)?;
    Ok(Json(pkgx))
});
