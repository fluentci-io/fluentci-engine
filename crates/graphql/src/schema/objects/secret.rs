use async_graphql::{Context, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Secret {
    pub id: ID,
}

#[Object]
impl Secret {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn plaintext(&self, ctx: &Context<'_>) -> &str {
        ""
    }
}
