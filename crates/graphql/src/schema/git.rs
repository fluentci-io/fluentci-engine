use std::sync::{Arc, Mutex};

use super::objects::git::Git;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct GitQuery;

#[Object]
impl GitQuery {
    async fn git(&self, ctx: &Context<'_>) -> Result<Git, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "git".into(),
            "".into(),
            vec![],
        ));

        drop(graph);

        let git = Git { id: ID(id) };
        Ok(git)
    }
}
