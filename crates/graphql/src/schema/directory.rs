use super::objects::{cache::Cache, directory::Directory};
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DirectoryQuery;

#[Object]
impl DirectoryQuery {
    async fn directory(&self, _ctx: &Context<'_>) -> Result<Directory, Error> {
        let directory = Directory { id: "".into() };
        Ok(directory)
    }
}
