use std::sync::{Arc, Mutex};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand, Output};
use uuid::Uuid;

use super::{devbox::Devbox, devenv::Devenv, flox::Flox, nix::Nix, pkgx::Pkgx};

#[derive(Debug, Clone, Default)]
pub struct Pipeline {
    pub id: ID,
}

#[Object]
impl Pipeline {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn devbox(&self, ctx: &Context<'_>) -> Result<Devbox, Error> {
        Ok(Devbox { id: "".into() })
    }

    async fn devenv(&self, ctx: &Context<'_>) -> Result<Devenv, Error> {
        Ok(Devenv { id: "".into() })
    }

    async fn flox(&self, ctx: &Context<'_>) -> Result<Flox, Error> {
        Ok(Flox { id: "".into() })
    }

    async fn nix(&self, ctx: &Context<'_>) -> Result<Nix, Error> {
        Ok(Nix { id: "".into() })
    }

    async fn pkgx(&self, ctx: &Context<'_>) -> Result<Pkgx, Error> {
        Ok(Pkgx { id: "".into() })
    }

    async fn with_exec(&self, ctx: &Context<'_>, args: Vec<String>) -> Result<&Pipeline, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "exec".into(),
            args.join(" "),
            deps,
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        Ok(self)
    }

    async fn with_work_dir(&self, path: String) -> Result<&Pipeline, Error> {
        Ok(self)
    }

    async fn with_service(&self, service: ID) -> Result<&Pipeline, Error> {
        Ok(self)
    }

    async fn with_cache(&self, cache: ID) -> Result<&Pipeline, Error> {
        Ok(self)
    }

    async fn stdout(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stdout));
        Ok("OK".to_string())
    }

    async fn stderr(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stderr));
        Ok("OK".to_string())
    }
}
