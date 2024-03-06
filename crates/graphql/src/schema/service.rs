use crate::schema::objects::service::Service;
use async_graphql::{Context, Error, Object};

#[derive(Default, Clone)]
pub struct ServiceQuery;

#[Object]
impl ServiceQuery {
    async fn service(&self, _ctx: &Context<'_>, name: String) -> Result<Service, Error> {
        let service = Service { id: name.into() };
        Ok(service.clone())
    }
}
