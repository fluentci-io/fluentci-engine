use async_graphql::{Error, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub id: ID,
}

#[Object]
impl Cache {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn key(&self) -> Result<String, Error> {
        Ok("".to_string())
    }

    async fn path(&self) -> Result<String, Error> {
        Ok("".to_string())
    }
}
