use super::objects::cache::Cache;
use async_graphql::{Context, Error, Object};

#[derive(Default, Clone)]
pub struct CacheQuery;

#[Object]
impl CacheQuery {
    async fn cache(&self, _ctx: &Context<'_>, key: String, path: String) -> Result<Cache, Error> {
        let cache = Cache { id: "".into() };
        Ok(cache.clone())
    }
}
