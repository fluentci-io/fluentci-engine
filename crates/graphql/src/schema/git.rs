use super::objects::git::Git;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct GitQuery;

#[Object]
impl GitQuery {
    async fn git(&self, _ctx: &Context<'_>) -> Result<Git, Error> {
        let git = Git { id: "".into() };
        Ok(git)
    }
}
