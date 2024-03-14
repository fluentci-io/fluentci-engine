use std::sync::{Arc, Mutex};

use super::objects::devenv::Devenv;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DevenvQuery;

#[Object]
impl DevenvQuery {
    async fn devenv(&self, ctx: &Context<'_>) -> Result<Devenv, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "devenv".into(),
            "".into(),
            vec![],
        ));

        drop(graph);

        let devenv = Devenv { id: ID(id) };
        Ok(devenv)
    }
}
