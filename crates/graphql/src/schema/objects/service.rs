use async_graphql::{Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Service {
    pub id: ID,
}

#[Object]
impl Service {
    async fn id(&self) -> &ID {
        &self.id
    }
}
