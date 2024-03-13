use async_graphql::{Context, Error, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Devenv {
    pub id: ID,
}

#[Object]
impl Devenv {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn with_exec(&self, ctx: &Context<'_>, args: Vec<String>) -> Result<&Devenv, Error> {
        Ok(self)
    }

    async fn with_work_dir(&self, path: String) -> Result<&Devenv, Error> {
        Ok(self)
    }

    async fn with_service(&self, service: ID) -> Result<&Devenv, Error> {
        Ok(self)
    }

    async fn with_cache(&self, cache: ID) -> Result<&Devenv, Error> {
        Ok(self)
    }

    async fn stdout(&self, ctx: &Context<'_>) -> Result<String, Error> {
        Ok("OK".to_string())
    }

    async fn stderr(&self, ctx: &Context<'_>) -> Result<String, Error> {
        Ok("OK".to_string())
    }
}
