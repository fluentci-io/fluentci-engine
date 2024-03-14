use std::sync::{mpsc::Receiver, Arc, Mutex};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand, Output};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct Devbox {
    pub id: ID,
}

#[Object]
impl Devbox {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn with_exec(&self, ctx: &Context<'_>, args: Vec<String>) -> Result<&Devbox, Error> {
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

    async fn with_work_dir(&self, path: String) -> Result<&Devbox, Error> {
        Ok(self)
    }

    async fn with_service(&self, service: ID) -> Result<&Devbox, Error> {
        Ok(self)
    }

    async fn with_cache(&self, cache: ID) -> Result<&Devbox, Error> {
        Ok(self)
    }

    async fn stdout(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stdout));
        let rx = ctx.data::<Arc<Mutex<Receiver<(String, usize)>>>>().unwrap();
        let rx = rx.lock().unwrap();
        let (stdout, code) = rx.recv().unwrap();
        drop(rx);
        drop(graph);

        if code != 0 {
            return Err(Error::new(format!(
                "Failed to execute command `{}`",
                stdout
            )));
        }

        Ok(stdout)
    }

    async fn stderr(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stderr));
        let rx = ctx.data::<Arc<Mutex<Receiver<(String, usize)>>>>().unwrap();
        let rx = rx.lock().unwrap();
        let (stderr, code) = rx.recv().unwrap();
        drop(rx);
        drop(graph);

        if code != 0 {
            return Err(Error::new(format!(
                "Failed to execute command `{}`",
                stderr
            )));
        }

        Ok(stderr)
    }
}
