import fluentci/dagger/directory.{type Directory}
import gleam/javascript/promise.{type Promise}

pub type GitModuleSource

@external(javascript, "../../dagger.mjs", "id")
pub fn id(module_source: GitModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "cloneURL")
pub fn clone_url(module_source: GitModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "commit")
pub fn commit(module_source: GitModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "contextDirectory")
pub fn context_directory(module_source: GitModuleSource) -> Directory

@external(javascript, "../../dagger.mjs", "htmlURL")
pub fn html_url(module_source: GitModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "root")
pub fn root(module_source: GitModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "rootSubpath")
pub fn root_subpath(module_source: GitModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "version")
pub fn version(module_source: GitModuleSource) -> Promise(String)
