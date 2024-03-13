use async_graphql::{Error, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Directory {
    pub id: ID,
}

#[Object]
impl Directory {
    async fn id(&self) -> &ID {
        &self.id
    }
}
