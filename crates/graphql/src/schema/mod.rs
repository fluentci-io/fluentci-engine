use async_graphql::MergedObject;

use self::{cache::CacheQuery, pipeline::PipelineQuery, service::ServiceQuery};

pub mod cache;
pub mod objects;
pub mod pipeline;
pub mod service;

#[derive(Default, MergedObject)]
pub struct Query(PipelineQuery, ServiceQuery, CacheQuery);
