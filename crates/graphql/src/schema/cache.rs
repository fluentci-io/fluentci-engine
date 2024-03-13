use super::objects::cache::Cache;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct CacheQuery;

#[Object]
impl CacheQuery {
    async fn cache(&self, _ctx: &Context<'_>, key: String, path: String) -> Result<Cache, Error> {
        let id = Uuid::new_v4().to_string();
        let cache = Cache {
            id: ID(id),
            key,
            path,
        };
        Ok(cache)
    }
}
