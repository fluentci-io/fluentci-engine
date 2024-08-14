import fluentci/dagger/container.{type Container}
import fluentci/dagger/file.{type File}
import fluentci/dagger/module.{type Module}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Directory

@external(javascript, "../../dagger.mjs", "id")
pub fn id(directory: Directory) -> Promise(String)

@external(javascript, "../../dagger.mjs", "asModule")
pub fn as_module(directory: Directory) -> Module

@external(javascript, "../../dagger.mjs", "diff")
pub fn diff(directory: Directory, other: Directory) -> Directory

@external(javascript, "../../dagger.mjs", "directory")
pub fn directory(d: Directory, path: String) -> Directory

@external(javascript, "../../dagger.mjs", "dockerBuild")
pub fn docker_build(directory: Directory) -> Container

@external(javascript, "../../dagger.mjs", "entries")
pub fn entries(directory: Directory) -> Promise(Array(String))

@external(javascript, "../../dagger.mjs", "export_")
pub fn export(directory: Directory, path: String) -> Promise(String)

@external(javascript, "../../dagger.mjs", "file")
pub fn file(directory: Directory, path: String) -> File

@external(javascript, "../../dagger.mjs", "glob")
pub fn glob(directory: Directory, pattern: String) -> Promise(Array(String))

@external(javascript, "../../dagger.mjs", "pipeline")
pub fn pipeline(directory: Directory, name: String) -> Directory

@external(javascript, "../../dagger.mjs", "sync")
pub fn sync(directory: Directory) -> Directory

@external(javascript, "../../dagger.mjs", "terminal")
pub fn terminal(directory: Directory) -> Directory

@external(javascript, "../../dagger.mjs", "withDirectory")
pub fn with_directory(
  d: Directory,
  path: String,
  directory: Directory,
) -> Directory

@external(javascript, "../../dagger.mjs", "withFile")
pub fn with_file(d: Directory, source: File) -> Directory

@external(javascript, "../../dagger.mjs", "withFiles")
pub fn with_files(d: Directory, sources: Array(File)) -> Directory

@external(javascript, "../../dagger.mjs", "withNewDirectory")
pub fn with_new_directory(d: Directory, path: String) -> Directory

@external(javascript, "../../dagger.mjs", "withNewFile")
pub fn with_new_file(d: Directory, path: String, contents: String) -> Directory

@external(javascript, "../../dagger.mjs", "withTimestamps")
pub fn with_timestamps(d: Directory, timestamp: Int) -> Directory

@external(javascript, "../../dagger.mjs", "withoutDirectory")
pub fn without_directory(d: Directory, path: String) -> Directory

@external(javascript, "../../dagger.mjs", "withoutFile")
pub fn without_file(d: Directory, path: String) -> Directory
