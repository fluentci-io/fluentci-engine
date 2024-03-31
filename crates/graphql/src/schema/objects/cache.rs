use async_graphql::{Object, ID};
use fluentci_types::cache as types;

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub id: ID,
    pub key: String,
}

#[Object]
impl Cache {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn key(&self) -> &str {
        &self.key
    }
}

impl From<types::Cache> for Cache {
    fn from(cache: types::Cache) -> Self {
        Self {
            id: ID(cache.id),
            key: cache.key,
        }
    }
}
