use std::{
    fs::metadata,
    path::Path,
    sync::{Arc, Mutex},
};

use super::objects::file::File;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::runner::Runner;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct FileQuery;

#[Object]
impl FileQuery {
    async fn file(&self, ctx: &Context<'_>, path: String) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(Runner::default()));
        graph.runner.setup()?;

        let id = Uuid::new_v4().to_string();

        let md = metadata(path.clone()).map_err(|e| Error::new(e.to_string()))?;
        if !md.is_file() {
            return Err(Error::new(format!("Path `{}` is not a file", path)));
        }

        if !Path::new(&path).exists() {
            return Err(Error::new(format!("File `{}` does not exist", path)));
        }

        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "file".into(),
            "".into(),
            vec![],
            Arc::new(Box::new(Runner::default()))
        ));

        let file = File { id: ID(id), path };
        Ok(file)
    }
}
