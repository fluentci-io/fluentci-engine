use std::sync::{Arc, Mutex};

use anyhow::Error;
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::cache::Cache as CacheExt;
use fluentci_types::cache::Cache;
use uuid::Uuid;

pub fn cache(graph: Arc<Mutex<Graph>>, key: &str) -> Result<Cache, Error> {
    let mut graph = graph.lock().unwrap();
    graph.reset();

    let cache = graph
        .vertices
        .iter()
        .find(|v| v.label == "cache" && v.command == key);

    if let Some(cache) = cache {
        return Ok(Cache {
            id: cache.id.clone(),
            key: cache.command.clone(),
        });
    }

    let id = Uuid::new_v4().to_string();
    graph.execute(GraphCommand::AddVertex(
        id.clone(),
        "cache".into(),
        key.into(),
        vec![],
        Arc::new(Box::new(CacheExt::default())),
    ));

    let cache = Cache {
        id,
        key: key.into(),
    };
    Ok(cache)
}
