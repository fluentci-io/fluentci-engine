use extism::{convert::Json, *};
use fluentci_common::nix::nix as common_nix;

use crate::state::State;

host_fn!(pub nix(user_data: State;) -> Json<Nix> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    let nix = common_nix(graph, true)?;
    Ok(Json(nix))
});
