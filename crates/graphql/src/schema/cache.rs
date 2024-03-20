use std::sync::{Arc, Mutex};

use super::objects::cache::Cache;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct CacheQuery;

#[Object]
impl CacheQuery {
    async fn cache(&self, ctx: &Context<'_>, key: String, path: String) -> Result<Cache, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "cache".into(),
            "".into(),
            vec![],
        ));

        let cache = Cache {
            id: ID(id),
            key,
            path,
        };
        Ok(cache)
    }
}
