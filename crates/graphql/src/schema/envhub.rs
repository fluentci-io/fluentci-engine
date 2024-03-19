use std::sync::{Arc, Mutex};

use super::objects::envhub::Envhub;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::envhub::Envhub as EnvhubExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct EnvhubQuery;

#[Object]
impl EnvhubQuery {
    async fn envhub(&self, ctx: &Context<'_>) -> Result<Envhub, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(EnvhubExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "envhub".into(),
            "".into(),
            vec![],
        ));

        let envhub = Envhub { id: ID(id) };
        Ok(envhub)
    }
}
