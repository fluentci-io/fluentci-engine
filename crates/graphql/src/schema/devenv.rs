use super::objects::devenv::Devenv;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DevenvQuery;

#[Object]
impl DevenvQuery {
    async fn devenv(&self, _ctx: &Context<'_>) -> Result<Devenv, Error> {
        let devenv = Devenv { id: "".into() };
        Ok(devenv)
    }
}
