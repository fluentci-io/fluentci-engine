use extism::{convert::Json, *};
use fluentci_common::file::{self, file as common_file};
use fluentci_types::directory::Directory;

use crate::state::State;

host_fn!(pub file(user_data: State; path: String) -> Json<File> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    let file = common_file(graph, path, true)?;
    Ok(Json(file))
});

host_fn!(pub unzip(user_data: State; path: String) -> Json<Directory> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    match file::unzip(graph, path, None) {
        Ok(directory) => Ok(Json(Directory::from(directory))),
        Err(e) => Err(e),
    }
});

host_fn!(pub tar_xzvf(user_data: State; path: String) -> Json<Directory> {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();

    match file::tar_xzvf(graph, path, None) {
        Ok(directory) => Ok(Json(Directory::from(directory))),
        Err(e) => Err(e),
    }
});

host_fn!(pub md5(user_data: State; path: String) -> String {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    file::md5(graph, path)
});

host_fn!(pub sha256(user_data: State; path: String) -> String {
    let state = user_data.get()?;
    let state = state.lock().unwrap();
    let graph = state.graph.clone();
    file::sha256(graph, path)
});
