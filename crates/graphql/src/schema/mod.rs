use async_graphql::MergedObject;

use self::{pipeline::PipelineQuery, service::ServiceQuery};

pub mod objects;
pub mod pipeline;
pub mod service;

#[derive(Default, MergedObject)]
pub struct Query(PipelineQuery, ServiceQuery);
