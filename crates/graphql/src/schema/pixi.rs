use std::sync::{Arc, Mutex};

use super::objects::pixi::Pixi;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::pixi::Pixi as PixiExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct PixiQuery;

#[Object]
impl PixiQuery {
    async fn pixi(&self, ctx: &Context<'_>) -> Result<Pixi, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(PixiExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "pixi".into(),
            "".into(),
            vec![],
            Arc::new(Box::new(PixiExt::default())),
        ));

        let pixi = Pixi { id: ID(id) };
        Ok(pixi)
    }
}
