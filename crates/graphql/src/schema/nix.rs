use std::sync::{Arc, Mutex};

use super::objects::nix::Nix;
use async_graphql::{Context, Error, Object};
use fluentci_common::nix::nix as common_nix;
use fluentci_core::deps::Graph;

#[derive(Default, Clone)]
pub struct NixQuery;

#[Object]
impl NixQuery {
    async fn nix(&self, ctx: &Context<'_>) -> Result<Nix, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let nix = common_nix(graph.clone(), true)?;
        Ok(Nix::from(nix))
    }
}
