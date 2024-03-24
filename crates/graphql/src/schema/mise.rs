use std::sync::{Arc, Mutex};

use super::objects::mise::Mise;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::mise::Mise as MiseExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct MiseQuery;

#[Object]
impl MiseQuery {
    async fn mise(&self, ctx: &Context<'_>) -> Result<Mise, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(MiseExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "mise".into(),
            "".into(),
            vec![],
            Arc::new(Box::new(MiseExt::default())),
        ));

        let mise = Mise { id: ID(id) };
        Ok(mise)
    }
}
