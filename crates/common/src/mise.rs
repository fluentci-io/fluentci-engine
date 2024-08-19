use std::{
    fs::canonicalize,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::mise::Mise as MiseExt;
use fluentci_types::mise::Mise;
use uuid::Uuid;

pub fn mise(graph: Arc<Mutex<Graph>>, reset: bool) -> Result<Mise, Error> {
    let mut graph = graph.lock().unwrap();

    if reset {
        graph.reset();
    }

    graph.runner = Arc::new(Box::new(MiseExt::default()));
    graph.runner.setup()?;

    let id = Uuid::new_v4().to_string();
    graph.execute(GraphCommand::AddVertex(
        id.clone(),
        "mise".into(),
        "".into(),
        vec![],
        Arc::new(Box::new(MiseExt::default())),
    ))?;

    let mise = Mise { id };
    Ok(mise)
}

pub fn trust(graph: Arc<Mutex<Graph>>, path: String) -> Result<(), Error> {
    let mut graph = graph.lock().unwrap();

    if !Path::new(&path).exists() && !path.starts_with("/") {
        let dir = canonicalize(".").unwrap();
        let dir = dir.to_str().unwrap();
        let dir = format!("{}/{}", dir, path);
        return Err(Error::msg(format!("Path `{}` does not exist", dir)));
    }

    if !Path::new(&path).exists() {
        return Err(Error::msg(format!("Path `{}` does not exist", path)));
    }

    let id = Uuid::new_v4().to_string();
    let dep_id = graph.vertices[graph.size() - 1].id.clone();
    let deps = match graph.size() {
        1 => vec![],
        _ => vec![dep_id],
    };
    graph.execute(GraphCommand::AddVertex(
        id.clone(),
        "trust".into(),
        format!("mise trust {}", path),
        deps,
        Arc::new(Box::new(MiseExt::default())),
    ))?;

    if graph.size() > 2 {
        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y))?;
    }

    graph.execute_vertex(&id)?;

    Ok(())
}
