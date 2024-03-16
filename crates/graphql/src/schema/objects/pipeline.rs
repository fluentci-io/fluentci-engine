use std::{
    fs::{self, canonicalize},
    path::Path,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::devbox::Devbox as DevboxExt;
use fluentci_ext::devenv::Devenv as DevenvExt;
use fluentci_ext::flox::Flox as FloxExt;
use fluentci_ext::git::Git as GitExt;
use fluentci_ext::http::Http as HttpExt;
use fluentci_ext::nix::Nix as NixExt;
use fluentci_ext::pkgx::Pkgx as PkgxExt;
use fluentci_types::Output;
use uuid::Uuid;

use crate::schema::objects::{file::File, git::Git};

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

    async fn http(&self, ctx: &Context<'_>, url: String) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(HttpExt::default()));
        graph.runner.setup()?;
        graph.work_dir = format!(
            "{}/.fluentci/cache",
            dirs::home_dir().unwrap().to_str().unwrap()
        );
        fs::create_dir_all(&graph.work_dir)?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "http".into(),
            url,
            vec![],
        ));
        graph.execute_vertex(&id)?;

        drop(graph);
        let file = File {
            id: ID(id),
            path: "/file".into(),
        };
        Ok(file)
    }

    async fn git(&self, ctx: &Context<'_>, url: String) -> Result<Git, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(GitExt::default()));
        graph.runner.setup()?;
        graph.work_dir = format!(
            "{}/.fluentci/cache",
            dirs::home_dir().unwrap().to_str().unwrap()
        );
        fs::create_dir_all(&graph.work_dir)?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "git".into(),
            url,
            vec![],
        ));
        graph.execute_vertex(&id)?;
        drop(graph);

        let git = Git { id: ID(id) };
        Ok(git)
    }

    async fn devbox(&self, ctx: &Context<'_>) -> Result<Devbox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(DevboxExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "devbox".into(),
            "".into(),
            vec![],
        ));

        let devbox = Devbox { id: ID(id) };
        Ok(devbox)
    }

    async fn devenv(&self, ctx: &Context<'_>) -> Result<Devenv, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(DevenvExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "devenv".into(),
            "".into(),
            vec![],
        ));

        let devenv = Devenv { id: ID(id) };
        Ok(devenv)
    }

    async fn flox(&self, ctx: &Context<'_>) -> Result<Flox, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(FloxExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "flox".into(),
            "".into(),
            vec![],
        ));

        let flox = Flox { id: ID(id) };
        Ok(flox)
    }

    async fn nix(&self, ctx: &Context<'_>) -> Result<Nix, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(NixExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "nix".into(),
            "".into(),
            deps,
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        let nix = Nix { id: ID(id) };
        Ok(nix)
    }

    async fn pkgx(&self, ctx: &Context<'_>) -> Result<Pkgx, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(PkgxExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "pkgx".into(),
            "".into(),
            vec![],
        ));

        let pkgx = Pkgx { id: ID(id) };
        Ok(pkgx)
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

    async fn with_work_dir(&self, ctx: &Context<'_>, path: String) -> Result<&Pipeline, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();

        if !Path::new(&path).exists() {
            let dir = canonicalize(".").unwrap();
            let dir = dir.to_str().unwrap();
            let dir = format!("{}/{}", dir, path);
            return Err(Error::new(format!("Path `{}` does not exist", dir)));
        }

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "withWorkdir".into(),
            path,
            deps,
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

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
