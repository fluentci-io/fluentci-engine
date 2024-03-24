use std::{
    fs,
    sync::{Arc, Mutex},
};

use super::objects::file::File;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::http::Http as HttpExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct HttpQuery;

#[Object]
impl HttpQuery {
    async fn http(&self, ctx: &Context<'_>, url: String) -> Result<File, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
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
            url.clone(),
            vec![],
            Arc::new(Box::new(HttpExt::default())),
        ));
        graph.execute_vertex(&id)?;

        let filename = sha256::digest(url).to_string();
        let work_dir = graph.work_dir.clone();

        let file = File {
            id: ID(id),
            path: format!("{}/{}", work_dir, filename),
        };
        Ok(file)
    }
}
