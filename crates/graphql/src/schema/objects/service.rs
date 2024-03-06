use async_graphql::{Error, Object, ID};

#[derive(Debug, Clone, Default)]
pub struct Service {
    pub id: ID,
}

#[Object]
impl Service {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn with_env_variable(&self, key: String, value: String) -> Result<&Service, Error> {
        Ok(self)
    }

    async fn with_cache(&self, key: String, path: String) -> Result<&Service, Error> {
        Ok(self)
    }

    async fn with_exec(&self, args: Vec<String>) -> Result<&Service, Error> {
        Ok(self)
    }

    async fn with_work_dir(&self, path: String) -> Result<&Service, Error> {
        Ok(self)
    }

    async fn with_service(&self, service: ID) -> Result<&Service, Error> {
        Ok(self)
    }

    async fn stdout(&self) -> Result<String, Error> {
        Ok("".to_string())
    }

    async fn stderr(&self) -> Result<String, Error> {
        Ok("".to_string())
    }
}
