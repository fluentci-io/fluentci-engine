use super::objects::pkgx::Pkgx;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct PkgxQuery;

#[Object]
impl PkgxQuery {
    async fn pkgx(&self, _ctx: &Context<'_>) -> Result<Pkgx, Error> {
        let pkgx = Pkgx { id: "".into() };
        Ok(pkgx)
    }
}
