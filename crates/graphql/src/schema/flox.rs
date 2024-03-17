use std::sync::{Arc, Mutex};

use super::objects::flox::Flox;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::flox::Flox as FloxExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct FloxQuery;

#[Object]
impl FloxQuery {
    async fn flox(&self, ctx: &Context<'_>) -> Result<Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(FloxExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "flox".into(),
            "".into(),
            vec![],
        ));

        let flox = Flox { id: ID(id) };
        Ok(flox)
    }
}
