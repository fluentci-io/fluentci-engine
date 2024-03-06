use super::objects::pipeline::Pipeline;
use async_graphql::{Context, Error, Object};

#[derive(Default, Clone)]
pub struct PipelineQuery;

#[Object]
impl PipelineQuery {
    async fn pipeline(&self, _ctx: &Context<'_>, name: String) -> Result<Pipeline, Error> {
        let pipeline = Pipeline { id: name.into() };
        Ok(pipeline.clone())
    }
}
