use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use schema::Query;

pub mod schema;

pub type FluentCISchema = Schema<Query, EmptyMutation, EmptySubscription>;
