use std::sync::{Arc, Mutex};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::git_checkout::GitCheckout as GitCheckoutExt;
use fluentci_ext::git_last_commit::GitLastCommit as GitLastCommitExt;
use fluentci_ext::runner::Runner as RunnerExt;
use uuid::Uuid;

use super::directory::Directory;

#[derive(Debug, Clone, Default)]
pub struct Git {
    pub id: ID,
}

#[Object]
impl Git {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn branch(&self, ctx: &Context<'_>, name: String) -> Result<&Git, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(GitCheckoutExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "git-checkout".into(),
            name,
            deps,
        ));
        graph.execute_vertex(&id)?;

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        Ok(&self)
    }

    async fn commit(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(GitLastCommitExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "git-last-commit".into(),
            "".into(),
            deps,
        ));
        graph.execute_vertex(&id)?;

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        Ok("".into())
    }

    async fn tree(&self, ctx: &Context<'_>) -> Result<Directory, Error> {
        let id = Uuid::new_v4().to_string();
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "tree".into(),
            "".into(),
            vec![dep_id],
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));
        graph.runner = Arc::new(Box::new(RunnerExt::default()));

        let directory = Directory {
            id: ID(id),
            path: graph.work_dir.clone(),
        };
        Ok(directory)
    }
}
