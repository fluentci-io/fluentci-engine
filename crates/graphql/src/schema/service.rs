use crate::schema::objects::service::Service;
use async_graphql::{Context, Error, Object, ID};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct ServiceQuery;

#[Object]
impl ServiceQuery {
    async fn service(&self, _ctx: &Context<'_>, name: String) -> Result<Service, Error> {
        let id = Uuid::new_v4().to_string();
        let service = Service { id: ID(id) };
        Ok(service.clone())
    }
}
