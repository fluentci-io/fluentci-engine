use async_graphql::{Error, Object, ID};
use uuid::Uuid;

use super::directory::Directory;

#[derive(Debug, Clone, Default)]
pub struct Git {
    pub id: ID,
}

#[Object]
impl Git {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn branch(&self) -> Result<&Git, Error> {
        Ok(&self)
    }

    async fn tree(&self) -> Result<Directory, Error> {
        let id = Uuid::new_v4().to_string();
        let directory = Directory { id: ID(id) };
        Ok(directory)
    }
}
