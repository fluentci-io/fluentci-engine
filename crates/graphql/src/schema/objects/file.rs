use async_graphql::{Error, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct File {
    pub id: ID,
    pub path: String,
}

#[Object]
impl File {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn path(&self) -> &str {
        &self.path
    }
}
