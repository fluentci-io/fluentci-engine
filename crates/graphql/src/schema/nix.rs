use super::objects::nix::Nix;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct NixQuery;

#[Object]
impl NixQuery {
    async fn nix(&self, _ctx: &Context<'_>) -> Result<Nix, Error> {
        let nix = Nix { id: "".into() };
        Ok(nix)
    }
}
