use std::sync::{Arc, Mutex};

use super::objects::{cache::Cache, directory::Directory};
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DirectoryQuery;

#[Object]
impl DirectoryQuery {
    async fn directory(&self, ctx: &Context<'_>) -> Result<Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "directory".into(),
            "".into(),
            vec![],
        ));

        drop(graph);

        let directory = Directory { id: ID(id) };
        Ok(directory)
    }
}
