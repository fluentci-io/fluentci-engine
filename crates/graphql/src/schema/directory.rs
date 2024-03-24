use std::{
    fs::{canonicalize, metadata},
    path::Path,
    sync::{Arc, Mutex},
};

use super::objects::directory::Directory;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::runner::Runner;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DirectoryQuery;

#[Object]
impl DirectoryQuery {
    async fn directory(&self, ctx: &Context<'_>, path: String) -> Result<Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(Runner::default()));
        graph.runner.setup()?;

        let md = metadata(path.clone()).map_err(|e| Error::new(e.to_string()))?;
        if !md.is_dir() {
            return Err(Error::new(format!("Path `{}` is not a directory", path)));
        }

        if !Path::new(&path).exists() && !path.starts_with("/") {
            let dir = canonicalize(".").unwrap();
            let dir = dir.to_str().unwrap();
            let dir = format!("{}/{}", dir, path);
            return Err(Error::new(format!("Path `{}` does not exist", dir)));
        }

        graph.work_dir = path.clone();

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "directory".into(),
            "".into(),
            vec![],
            Arc::new(Box::new(Runner::default())),
        ));

        let path = canonicalize(path).unwrap().to_str().unwrap().to_string();
        let directory = Directory { id: ID(id), path };
        Ok(directory)
    }
}
