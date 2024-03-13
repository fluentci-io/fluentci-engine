use async_graphql::{Error, Object, ID};

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
        let directory = Directory { id: "".into() };
        Ok(directory)
    }
}
