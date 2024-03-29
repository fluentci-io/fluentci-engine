use std::{
    fs::canonicalize,
    path::Path,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};

use fluentci_ext::archive::zip::Zip as ZipExt;
use fluentci_ext::cache::Cache as CacheExt;
use fluentci_ext::devbox::Devbox as DevboxExt;
use fluentci_ext::devenv::Devenv as DevenvExt;
use fluentci_ext::envhub::Envhub as EnvhubExt;
use fluentci_ext::flox::Flox as FloxExt;
use fluentci_ext::mise::Mise as MiseExt;
use fluentci_ext::nix::Nix as NixExt;
use fluentci_ext::pixi::Pixi as PixiExt;
use fluentci_ext::pkgx::Pkgx as PkgxExt;
use fluentci_ext::{archive::tar::czvf::TarCzvf as TarCzvfExt, runner::Runner};
use fluentci_types::Output;
use uuid::Uuid;

use crate::schema::objects::{envhub::Envhub, file::File, mise::Mise};

use super::{devbox::Devbox, devenv::Devenv, flox::Flox, nix::Nix, pixi::Pixi, pkgx::Pkgx};

#[derive(Debug, Clone, Default)]
pub struct Directory {
    pub id: ID,
    pub path: String,
}

#[Object]
impl Directory {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn path(&self) -> &str {
        &self.path
    }

    async fn directory(&self, path: String) -> Result<Directory, Error> {
        let id = Uuid::new_v4().to_string();
        let directory = Directory { id: ID(id), path };
        Ok(directory)
    }

    async fn entries(&self) -> Result<Vec<String>, Error> {
        let path = self.path.clone();

        if !Path::new(&path).exists() {
            return Err(Error::new(format!("Path `{}` does not exist", path)));
        }

        let entries = tokio::task::spawn_blocking(move || {
            let entries = std::fs::read_dir(&path)
                .unwrap()
                .map(|res| res.unwrap().file_name().into_string().unwrap())
                .collect();
            entries
        })
        .await
        .unwrap();
        Ok(entries)
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
            Arc::new(Box::new(DevboxExt::default())),
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
            Arc::new(Box::new(DevenvExt::default())),
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
            Arc::new(Box::new(FloxExt::default())),
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
            Arc::new(Box::new(NixExt::default())),
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
            Arc::new(Box::new(PkgxExt::default())),
        ));

        let pkgx = Pkgx { id: ID(id) };
        Ok(pkgx)
    }

    async fn pixi(&self, ctx: &Context<'_>) -> Result<Pixi, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(PixiExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "pixi".into(),
            "".into(),
            deps,
            Arc::new(Box::new(PixiExt::default())),
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        let pixi = Pixi { id: ID(id) };
        Ok(pixi)
    }

    async fn mise(&self, ctx: &Context<'_>) -> Result<Mise, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(MiseExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "mise".into(),
            "".into(),
            deps,
            Arc::new(Box::new(MiseExt::default())),
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        let mise = Mise { id: ID(id) };
        Ok(mise)
    }

    async fn envhub(&self, ctx: &Context<'_>) -> Result<Envhub, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(EnvhubExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let dep_id = graph.vertices[graph.size() - 1].id.clone();
        let deps = match graph.size() {
            1 => vec![],
            _ => vec![dep_id],
        };
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "envhub".into(),
            "".into(),
            deps,
            Arc::new(Box::new(EnvhubExt::default())),
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        let envhub = Envhub { id: ID(id) };
        Ok(envhub)
    }

    async fn with_workdir(&self, ctx: &Context<'_>, path: String) -> Result<&Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();

        if !Path::new(&path).exists() {
            let dir = canonicalize(".").unwrap();
            let dir = dir.to_str().unwrap();
            let dir = format!("{}/{}", dir, path);
            return Err(Error::new(format!("Path `{}` does not exist", dir)));
        }

        if !Path::new(&path).exists() {
            return Err(Error::new(format!("Path `{}` does not exist", path)));
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
            Arc::new(Box::new(Runner::default())),
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        Ok(self)
    }

    async fn with_exec(&self, ctx: &Context<'_>, args: Vec<String>) -> Result<&Directory, Error> {
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
            Arc::new(Box::new(Runner::default())),
        ));

        if graph.size() > 2 {
            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));
        }

        Ok(self)
    }

    async fn with_service(&self, _service: ID) -> Result<&Directory, Error> {
        Ok(self)
    }

    async fn with_cache(
        &self,
        ctx: &Context<'_>,
        path: String,
        cache_id: ID,
    ) -> Result<&Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        let runner = graph.runner.clone();
        graph.runner = Arc::new(Box::new(CacheExt::default()));
        graph.runner.setup()?;

        if let Some(cache) = graph.vertices.iter().find(|v| ID(v.id.clone()) == cache_id) {
            let id = Uuid::new_v4().to_string();
            let dep_id = graph.vertices[graph.size() - 1].id.clone();
            let deps = match graph.size() {
                1 => vec![],
                _ => vec![dep_id],
            };
            let cache_key_path = format!("{}:{}", cache.command, path);
            graph.execute(GraphCommand::AddVertex(
                id.clone(),
                "withCache".into(),
                cache_key_path,
                deps,
                Arc::new(Box::new(CacheExt::default())),
            ));

            let x = graph.size() - 2;
            let y = graph.size() - 1;
            graph.execute(GraphCommand::AddEdge(x, y));

            graph.execute_vertex(&id)?;
            graph.runner = runner;
            return Ok(self);
        }

        return Err(Error::new("Cache not found"));
    }

    async fn stdout(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.execute(GraphCommand::Execute(Output::Stdout));
        let rx = ctx.data::<Arc<Mutex<Receiver<(String, usize)>>>>().unwrap();
        let rx = rx.lock().unwrap();
        let (stdout, code) = rx.recv().unwrap();

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

        if code != 0 {
            return Err(Error::new(format!(
                "Failed to execute command `{}`",
                stderr
            )));
        }

        Ok(stderr)
    }

    async fn zip(&self, ctx: &Context<'_>) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(ZipExt::default()));
        graph.runner.setup()?;

        let parent_dir = self.path.split('/').collect::<Vec<&str>>();
        let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");
        graph.work_dir = parent_dir.clone();

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "zip".into(),
            self.path.clone(),
            vec![dep_id],
            Arc::new(Box::new(ZipExt::default())),
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        graph.execute_vertex(&id)?;

        let output_file = match self.path.split('/').last() {
            Some(file) => format!("{}.zip", file),
            None => format!("{}.zip", self.path),
        };

        let file = File {
            id: ID(id),
            path: format!("{}/{}", parent_dir, output_file),
        };
        Ok(file)
    }

    async fn tar_czvf(&self, ctx: &Context<'_>) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(TarCzvfExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "tar czvf".into(),
            self.path.clone(),
            vec![dep_id],
            Arc::new(Box::new(TarCzvfExt::default())),
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        graph.execute_vertex(&id)?;

        let output_file = match self.path.split('/').last() {
            Some(file) => format!("{}.tar.gz", file),
            None => format!("{}.tar.gz", self.path),
        };

        let parent_dir = self.path.split('/').collect::<Vec<&str>>();
        let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");

        let file = File {
            id: ID(id),
            path: format!("{}/{}", parent_dir, output_file),
        };
        Ok(file)
    }
}
