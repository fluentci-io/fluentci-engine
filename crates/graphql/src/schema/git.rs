use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::util::{extract_git_repo, validate_git_url};

use super::objects::git::Git;
use async_graphql::{Context, Error, Object, ID};
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::git::Git as GitExt;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct GitQuery;

#[Object]
impl GitQuery {
    async fn git(&self, ctx: &Context<'_>, url: String) -> Result<Git, Error> {
        let graph = ctx.data::<Arc<Mutex<Graph>>>().unwrap();
        let mut graph = graph.lock().unwrap();
        graph.reset();
        graph.runner = Arc::new(Box::new(GitExt::default()));
        graph.runner.setup()?;
        graph.work_dir = format!(
            "{}/.fluentci/cache",
            dirs::home_dir().unwrap().to_str().unwrap()
        );

        if !validate_git_url(&url) {
            return Err(Error::new("Invalid git url"));
        }
        let repo = extract_git_repo(&url);
        graph.work_dir = format!("{}/{}", graph.work_dir, repo);

        fs::create_dir_all(&graph.work_dir)?;

        let id = Uuid::new_v4().to_string();
        graph.execute(GraphCommand::AddVertex(
            id.clone(),
            "git".into(),
            url.clone(),
            vec![],
        ));
        graph.execute_vertex(&id)?;
        graph.work_dir = format!(
            "{}/{}",
            graph.work_dir,
            url.split("/").last().unwrap().replace(".git", "")
        );
        let git = Git { id: ID(id) };
        Ok(git)
    }
}
