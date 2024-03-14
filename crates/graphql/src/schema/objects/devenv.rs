use std::{
    process::ExitStatus,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand, Output};
use uuid::Uuid;

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
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stdout));
        drop(graph);
        let rx = ctx
            .data::<Arc<Mutex<Receiver<(String, ExitStatus)>>>>()
            .unwrap();
        let rx = rx.lock().unwrap();
        let (stdout, _) = rx.recv().unwrap();
        drop(rx);
        Ok(stdout)
    }

    async fn stderr(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stderr));
        let rx = ctx
            .data::<Arc<Mutex<Receiver<(String, ExitStatus)>>>>()
            .unwrap();
        let rx = rx.lock().unwrap();
        let (stderr, _) = rx.recv().unwrap();
        Ok(stderr)
    }
}
