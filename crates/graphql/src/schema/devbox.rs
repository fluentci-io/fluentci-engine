use super::objects::devbox::Devbox;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DevboxQuery;

#[Object]
impl DevboxQuery {
    async fn devbox(&self, _ctx: &Context<'_>) -> Result<Devbox, Error> {
        let devbox = Devbox { id: "".into() };
        Ok(devbox)
    }
}
