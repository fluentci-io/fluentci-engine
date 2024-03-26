use std::sync::{Arc, Mutex};

use anyhow::Error;
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::pkgx::Pkgx as PkgxExt;
use fluentci_types::pkgx::Pkgx;
use uuid::Uuid;

pub fn pkgx(graph: Arc<Mutex<Graph>>, reset: bool) -> Result<Pkgx, Error> {
    let mut graph = graph.lock().unwrap();

    if reset {
        graph.reset();
    }

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

    let pkgx = Pkgx { id };
    Ok(pkgx)
}
