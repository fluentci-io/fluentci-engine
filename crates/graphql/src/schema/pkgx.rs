use std::sync::{Arc, Mutex};

use super::objects::pkgx::Pkgx;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::pkgx::Pkgx as PkgxExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct PkgxQuery;

#[Object]
impl PkgxQuery {
    async fn pkgx(&self, ctx: &Context<'_>) -> Result<Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(PkgxExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "pkgx".into(),
            "".into(),
            vec![],
            Arc::new(Box::new(PkgxExt::default())),
        ));

        let pkgx = Pkgx { id: ID(id) };
        Ok(pkgx)
    }
}
