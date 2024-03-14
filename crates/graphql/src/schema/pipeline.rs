use std::sync::{Arc, Mutex};

use super::objects::pipeline::Pipeline;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct PipelineQuery;

#[Object]
impl PipelineQuery {
    async fn pipeline(&self, ctx: &Context<'_>, name: String) -> Result<Pipeline, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            name.into(),
            "".into(),
            vec![],
        ));

        drop(graph);
        let pipeline = Pipeline { id: ID(id) };
        Ok(pipeline)
    }
}
