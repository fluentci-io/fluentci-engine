use super::objects::flox::Flox;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct FloxQuery;

#[Object]
impl FloxQuery {
    async fn flox(&self, _ctx: &Context<'_>) -> Result<Flox, Error> {
        let flox = Flox { id: "".into() };
        Ok(flox)
    }
}
