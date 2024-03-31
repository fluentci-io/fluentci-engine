use std::sync::{Arc, Mutex};

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
    ));

    let mise = Mise { id };
    Ok(mise)
}
