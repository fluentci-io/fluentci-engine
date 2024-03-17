use std::sync::{Arc, Mutex};

use super::objects::devbox::Devbox;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::devbox::Devbox as DevboxExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DevboxQuery;

#[Object]
impl DevboxQuery {
    async fn devbox(&self, ctx: &Context<'_>) -> Result<Devbox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(DevboxExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "devbox".into(),
            "".into(),
            vec![],
        ));

        let devbox = Devbox { id: ID(id) };
        Ok(devbox)
    }
}
