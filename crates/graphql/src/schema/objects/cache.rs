use async_graphql::{Object, ID};

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
