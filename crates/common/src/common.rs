use std::{
    fs::canonicalize,
    path::Path,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use anyhow::Error;
use fluentci_core::deps::{Graph, GraphCommand};
use fluentci_ext::archive::tar::czvf::TarCzvf as TarCzvfExt;
use fluentci_ext::archive::zip::Zip as ZipExt;
use fluentci_ext::cache::Cache as CacheExt;
use fluentci_ext::Extension;
use fluentci_types::{file::File, Output};
use uuid::Uuid;

pub fn with_exec(
    graph: Arc<Mutex<Graph>>,
    args: Vec<String>,
    ext: Arc<Box<dyn Extension + Send + Sync>>,
) {
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
        ext,
    ));

    if graph.size() > 2 {
        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));
    }
}

pub fn with_workdir(
    graph: Arc<Mutex<Graph>>,
    path: String,
    ext: Arc<Box<dyn Extension + Send + Sync>>,
) -> Result<(), Error> {
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
        "withWorkdir".into(),
        path,
        deps,
        ext,
    ));

    if graph.size() > 2 {
        let x = graph.size() - 2;
        let y = graph.size() - 1;
        graph.execute(GraphCommand::AddEdge(x, y));
    }

    Ok(())
}

pub fn with_cache(graph: Arc<Mutex<Graph>>, cache_id: String, path: String) -> Result<(), Error> {
    let mut graph = graph.lock().unwrap();
    let runner = graph.runner.clone();
    graph.runner = Arc::new(Box::new(CacheExt::default()));
    graph.runner.setup()?;

    if let Some(cache) = graph.vertices.iter().find(|v| v.id.clone() == cache_id) {
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
        return Ok(());
    }

    return Err(Error::msg("Cache not found"));
}

pub fn stdout(
    graph: Arc<Mutex<Graph>>,
    rx: Arc<Mutex<Receiver<(String, usize)>>>,
) -> Result<String, Error> {
    let mut graph = graph.lock().unwrap();
    graph.execute(GraphCommand::Execute(Output::Stdout));
    let rx = rx.lock().unwrap();
    let (stdout, code) = rx.recv().unwrap();

    if code != 0 {
        return Err(Error::msg(format!(
            "Failed to execute command `{}`",
            stdout
        )));
    }
    Ok(stdout)
}

pub fn stderr(
    graph: Arc<Mutex<Graph>>,
    rx: Arc<Mutex<Receiver<(String, usize)>>>,
) -> Result<String, Error> {
    let mut graph = graph.lock().unwrap();
    graph.execute(GraphCommand::Execute(Output::Stderr));
    let rx = rx.lock().unwrap();
    let (stderr, code) = rx.recv().unwrap();

    if code != 0 {
        return Err(Error::msg(format!(
            "Failed to execute command `{}`",
            stderr
        )));
    }

    Ok(stderr)
}

pub fn zip(graph: Arc<Mutex<Graph>>, path: String) -> Result<File, Error> {
    let mut graph = graph.lock().unwrap();
    graph.runner = Arc::new(Box::new(ZipExt::default()));
    graph.runner.setup()?;

    let id = Uuid::new_v4().to_string();
    let dep_id = graph.vertices[graph.size() - 1].id.clone();

    graph.execute(GraphCommand::AddVertex(
        id.clone(),
        "zip".into(),
        path.clone(),
        vec![dep_id],
        Arc::new(Box::new(ZipExt::default())),
    ));

    let x = graph.size() - 2;
    let y = graph.size() - 1;
    graph.execute(GraphCommand::AddEdge(x, y));

    graph.execute_vertex(&id)?;

    let output_file = match path.split('/').last() {
        Some(file) => format!("{}.zip", file),
        None => format!("{}.zip", path),
    };

    let parent_dir = path.split('/').collect::<Vec<&str>>();
    let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");

    let file = File {
        id,
        path: format!("{}/{}", parent_dir, output_file),
    };

    Ok(file)
}

pub fn tar_czvf(graph: Arc<Mutex<Graph>>, path: String) -> Result<File, Error> {
    let mut graph = graph.lock().unwrap();
    graph.runner = Arc::new(Box::new(TarCzvfExt::default()));
    graph.runner.setup()?;

    let id = Uuid::new_v4().to_string();
    let dep_id = graph.vertices[graph.size() - 1].id.clone();

    graph.execute(GraphCommand::AddVertex(
        id.clone(),
        "tar czvf".into(),
        path.clone(),
        vec![dep_id],
        Arc::new(Box::new(TarCzvfExt::default())),
    ));

    let x = graph.size() - 2;
    let y = graph.size() - 1;
    graph.execute(GraphCommand::AddEdge(x, y));

    graph.execute_vertex(&id)?;

    let output_file = match path.split('/').last() {
        Some(file) => format!("{}.tar.gz", file),
        None => format!("{}.tar.gz", path),
    };

    let parent_dir = path.split('/').collect::<Vec<&str>>();
    let parent_dir = parent_dir[..parent_dir.len() - 1].join("/");

    let file = File {
        id,
        path: format!("{}/{}", parent_dir, output_file),
    };
    Ok(file)
}
