use std::env;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::archive::tar::{czvf::TarCzvf as TarCzvfExt, xzvf::TarXzvf as TarXzvfExt};
use fluentci_ext::archive::unzip::Unzip as UnzipExt;
use fluentci_ext::archive::zip::Zip as ZipExt;
use fluentci_ext::hash::md5::Md5 as Md5Ext;
use fluentci_ext::hash::sha256::Sha256 as Sha256Ext;
use fluentci_types::Output;
use uuid::Uuid;

use crate::schema::objects::directory::Directory;

#[derive(Debug, Clone, Default)]
pub struct File {
    pub id: ID,
    pub path: String,
}

#[Object]
impl File {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn path(&self) -> &str {
        &self.path
    }

    async fn zip(&self, ctx: &Context<'_>) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(ZipExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "zip".into(),
            self.path.clone(),
            vec![dep_id],
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        graph.execute_vertex(&id)?;

        let output_file = match self.path.split('/').last() {
            Some(file) => format!("{}.zip", file),
            None => format!("{}.zip", self.path),
        };

        let parent_dir = self.path.split('/').collect::<Vec<&str>>();
        let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");

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

    async fn unzip(
        &self,
        ctx: &Context<'_>,
        output_dir: Option<String>,
    ) -> Result<Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(UnzipExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "unzip".into(),
            self.path.clone(),
            vec![dep_id],
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        let output_dir = match output_dir {
            Some(dir) => dir,
            None => ".".into(),
        };

        env::set_var("FLUENTCI_UNZIP_OUTPUT_DIRECTORY", output_dir.clone());

        graph.execute_vertex(&id)?;

        let parent_dir = self.path.split('/').collect::<Vec<&str>>();
        let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");

        let dir = Directory {
            id: ID(id),
            path: format!("{}/{}", parent_dir, output_dir),
        };
        Ok(dir)
    }

    async fn tar_xzvf(
        &self,
        ctx: &Context<'_>,
        output_dir: Option<String>,
    ) -> Result<Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(TarXzvfExt::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "tar xzvf".into(),
            self.path.clone(),
            vec![dep_id],
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        let output_dir = match output_dir {
            Some(dir) => dir,
            None => ".".into(),
        };

        env::set_var("FLUENTCI_TAR_XZVF_OUTPUT_DIRECTORY", output_dir.clone());

        graph.execute_vertex(&id)?;

        let parent_dir = self.path.split('/').collect::<Vec<&str>>();
        let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");

        let dir = Directory {
            id: ID(id),
            path: format!("{}/{}", parent_dir, output_dir),
        };
        Ok(dir)
    }

    async fn md5(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(Md5Ext::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "md5".into(),
            self.path.clone(),
            vec![dep_id],
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        let hash = graph.execute_vertex(&id)?;

        Ok(hash)
    }

    async fn sha256(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.runner = Arc::new(Box::new(Sha256Ext::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();
        let dep_id = graph.vertices[graph.size() - 1].id.clone();

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "sha256".into(),
            self.path.clone(),
            vec![dep_id],
        ));

        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));

        let hash = graph.execute_vertex(&id)?;

        Ok(hash)
    }
}
