use std::sync::{mpsc::Receiver, Arc, Mutex};

use async_graphql::{Context, Error, Object, ID};
use fluentci_common::common;
use fluentci_core::deps::Graph;
use fluentci_ext::flox::Flox as FloxExt;
use fluentci_types::flox as types;

use super::service::Service;

#[derive(Debug, Clone, Default)]
pub struct Flox {
    pub id: ID,
}

#[Object]
impl Flox {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn with_exec(&self, ctx: &Context<'_>, args: Vec<String>) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_exec(graph.clone(), args, Arc::new(Box::new(FloxExt::default())));
        Ok(self)
    }

    async fn with_workdir(&self, ctx: &Context<'_>, path: String) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_workdir(graph.clone(), path, Arc::new(Box::new(FloxExt::default())))?;
        Ok(self)
    }

    async fn with_service(&self, ctx: &Context<'_>, service_id: ID) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_service(graph.clone(), service_id.into())?;
        Ok(self)
    }

    async fn with_cache(
        &self,
        ctx: &Context<'_>,
        path: String,
        cache_id: ID,
    ) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_cache(graph.clone(), cache_id.into(), path)?;
        Ok(self)
    }

    async fn with_file(
        &self,
        ctx: &Context<'_>,
        path: String,
        file_id: ID,
    ) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_file(graph.clone(), file_id.into(), path)?;
        Ok(self)
    }

    async fn stdout(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let rx = ctx.data::<Arc<Mutex<Receiver<(String, usize)>>>>().unwrap();
        common::stdout(graph.clone(), rx.clone()).map_err(|e| Error::new(e.to_string()))
    }

    async fn stderr(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let rx = ctx.data::<Arc<Mutex<Receiver<(String, usize)>>>>().unwrap();
        common::stderr(graph.clone(), rx.clone()).map_err(|e| Error::new(e.to_string()))
    }

    async fn as_service(&self, ctx: &Context<'_>, name: String) -> Result<Service, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let service = common::as_service(graph.clone(), name)?;
        Ok(service.into())
    }

    async fn with_env_variable(
        &self,
        ctx: &Context<'_>,
        name: String,
        value: String,
    ) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_env_variable(graph.clone(), &name, &value)?;
        Ok(self)
    }

    async fn wait_on(
        &self,
        ctx: &Context<'_>,
        port: u32,
        timeout: Option<u32>,
    ) -> Result<&Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::wait_on(graph.clone(), port, timeout)?;
        Ok(self)
    }
}

impl From<types::Flox> for Flox {
    fn from(flox: types::Flox) -> Self {
        Self { id: ID(flox.id) }
    }
}
