use std::{
    fs::canonicalize,
    path::Path,
    sync::{Arc, Mutex},
};

use super::objects::directory::Directory;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct DirectoryQuery;

#[Object]
impl DirectoryQuery {
    async fn directory(&self, ctx: &Context<'_>, path: String) -> Result<Directory, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();

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
        ));

        let directory = Directory { id: ID(id), path };
        Ok(directory)
    }
}
