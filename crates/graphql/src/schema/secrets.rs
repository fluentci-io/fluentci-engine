use std::sync::{Arc, Mutex};

use async_graphql::{Context, Error, Object};
use fluentci_core::deps::Graph;

use crate::schema::objects::secret_manager::SecretManager;

#[derive(Default, Clone)]
pub struct SecretsQuery;

#[Object]
impl SecretsQuery {
    async fn google_cloud_secret_manager(&self, ctx: &Context<'_>) -> Result<SecretManager, Error> {
        let _graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        Ok(SecretManager {
            id: "google_cloud_secret_manager".into(),
        })
    }

    async fn aws_secrets_manager(&self, ctx: &Context<'_>) -> Result<SecretManager, Error> {
        let _graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        Ok(SecretManager {
            id: "aws_secrets_manager".into(),
        })
    }

    async fn azure_keyvault(&self, ctx: &Context<'_>) -> Result<SecretManager, Error> {
        let _graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        Ok(SecretManager {
            id: "azure_keyvault".into(),
        })
    }

    async fn hashicorp_vault(&self, ctx: &Context<'_>) -> Result<SecretManager, Error> {
        let _graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        Ok(SecretManager {
            id: "hashicorp_vault".into(),
        })
    }
}
