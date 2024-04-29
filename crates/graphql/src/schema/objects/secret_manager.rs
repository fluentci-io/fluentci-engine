use async_graphql::{Context, Error, Object, ID};

use crate::schema::objects::secret::Secret;

#[derive(Debug, Clone, Default)]
pub struct SecretManager {
    pub id: ID,
}

#[Object]
impl SecretManager {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn get_secret(&self, ctx: &Context<'_>, path: String) -> Result<Secret, Error> {
        Ok(Secret {
            id: ID("".to_string()),
        })
    }
}
