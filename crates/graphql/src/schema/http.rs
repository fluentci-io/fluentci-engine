use std::sync::{Arc, Mutex};

use super::objects::file::File;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct HttpQuery;

#[Object]
impl HttpQuery {
    async fn http(&self, ctx: &Context<'_>) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "http".into(),
            "".into(),
            vec![],
        ));

        drop(graph);
        let file = File { id: ID(id) };
        Ok(file)
    }
}
