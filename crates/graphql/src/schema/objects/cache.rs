use async_graphql::{Error, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub id: ID,
    pub key: String,
    pub path: String,
}

#[Object]
impl Cache {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn key(&self) -> &str {
        &self.key
    }

    async fn path(&self) -> &str {
        &self.path
    }
}
