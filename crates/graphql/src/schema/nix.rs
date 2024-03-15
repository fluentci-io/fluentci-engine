use std::sync::{Arc, Mutex};

use super::objects::nix::Nix;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::nix::Nix as NixExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct NixQuery;

#[Object]
impl NixQuery {
    async fn nix(&self, ctx: &Context<'_>) -> Result<Nix, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(NixExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "nix".into(),
            "".into(),
            vec![],
        ));

        drop(graph);

        let nix = Nix { id: ID(id) };
        Ok(nix)
    }
}
