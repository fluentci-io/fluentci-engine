use super::objects::{cache::Cache, directory::Directory, file::File};
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct FileQuery;

#[Object]
impl FileQuery {
    async fn file(&self, _ctx: &Context<'_>) -> Result<File, Error> {
        let file = File { id: "".into() };
        Ok(file)
    }
}
