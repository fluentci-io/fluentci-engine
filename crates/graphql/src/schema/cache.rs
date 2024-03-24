use std::sync::{Arc, Mutex};

use super::objects::cache::Cache;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::cache::Cache as CacheExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct CacheQuery;

#[Object]
impl CacheQuery {
    async fn cache(&self, ctx: &Context<'_>, key: String) -> Result<Cache, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();

        let cache = graph
            .vertices
            .iter()
            .find(|v| v.label == "cache" && v.command == key);

        if let Some(cache) = cache {
            return Ok(Cache {
                id: ID(cache.id.clone()),
                key: cache.command.clone(),
            });
        }

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "cache".into(),
            key.clone(),
            vec![],
            Arc::new(Box::new(CacheExt::default())),
        ));

        let cache = Cache { id: ID(id), key };
        Ok(cache)
    }
}
