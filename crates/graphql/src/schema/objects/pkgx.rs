use std::sync::{mpsc::Receiver, Arc, Mutex};

use async_graphql::{Context, Error, Object, ID};
use fluentci_common::{common, pkgx as common_pkgx};
use fluentci_core::deps::Graph;
use fluentci_ext::pkgx::Pkgx as PkgxExt;
use fluentci_types::pkgx as types;

#[derive(Debug, Clone, Default)]
pub struct Pkgx {
    pub id: ID,
}

#[Object]
impl Pkgx {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn with_exec(&self, ctx: &Context<'_>, args: Vec<String>) -> Result<&Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_exec(graph.clone(), args, Arc::new(Box::new(PkgxExt::default())));
        Ok(self)
    }

    async fn with_workdir(&self, ctx: &Context<'_>, path: String) -> Result<&Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_workdir(graph.clone(), path, Arc::new(Box::new(PkgxExt::default())))?;
        Ok(self)
    }

    async fn with_service(&self, _service: ID) -> Result<&Pkgx, Error> {
        Ok(self)
    }

    async fn with_cache(
        &self,
        ctx: &Context<'_>,
        path: String,
        cache_id: ID,
    ) -> Result<&Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_cache(graph.clone(), cache_id.into(), path)?;
        Ok(self)
    }

    async fn with_file(
        &self,
        ctx: &Context<'_>,
        path: String,
        file_id: ID,
    ) -> Result<&Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common::with_file(graph.clone(), file_id.into(), path)?;
        Ok(self)
    }

    async fn with_packages(
        &self,
        ctx: &Context<'_>,
        packages: Vec<String>,
    ) -> Result<&Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        common_pkgx::with_packages(graph.clone(), packages)?;
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

    async fn as_service(&self, ctx: &Context<'_>) -> Result<ID, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let graph = graph.lock().unwrap();
        let id = graph.vertices[graph.size() - 1].id.clone();
        Ok(ID(id))
    }
}

impl From<types::Pkgx> for Pkgx {
    fn from(pkgx: types::Pkgx) -> Self {
        Self { id: ID(pkgx.id) }
    }
}
