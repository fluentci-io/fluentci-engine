//! A complete plan is one guest effect: matrix members never share a mutable graph.
use crate::state::State;
use extism::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    plan: fluentci_executor::Plan,
    run_dir: PathBuf,
}

host_fn!(pub execute_plan(user_data: State; request: String) -> String {
    let request: Request = serde_json::from_str(&request)?;
    let cancellation = {
        let state = user_data.get()?;
        let state = state.lock().unwrap();
        let graph = state.graph.lock().unwrap();
        graph.cancellation.clone()
    };
    let report = fluentci_core::execute_plan(&request.plan, &request.run_dir, &cancellation)?;
    Ok(serde_json::to_string(&report)?)
});
