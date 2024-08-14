import fluentci/dagger/directory.{type Directory}
import fluentci/dagger/git_module_source.{type GitModuleSource}
import fluentci/dagger/local_module_source.{type LocalModuleSource}
import fluentci/dagger/module.{
  type Module, type ModuleDependency, type ModuleSource, type ModuleSourceView,
}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

@external(javascript, "../../dagger.mjs", "id")
pub fn id(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "asGitSource")
pub fn as_git_source(module_source: ModuleSource) -> GitModuleSource

@external(javascript, "../../dagger.mjs", "asLocalSource")
pub fn as_local_source(module_source: ModuleSource) -> LocalModuleSource

@external(javascript, "../../dagger.mjs", "asModule")
pub fn as_module(module_source: ModuleSource) -> Module

@external(javascript, "../../dagger.mjs", "asString")
pub fn as_string(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "configExists")
pub fn config_exists(module_source: ModuleSource) -> Promise(bool)

@external(javascript, "../../dagger.mjs", "contextDirectory")
pub fn context_directory(module_source: ModuleSource) -> Directory

@external(javascript, "../../dagger.mjs", "dependencies")
pub fn dependencies(
  module_source: ModuleSource,
) -> Promise(Array(ModuleDependency))

@external(javascript, "../../dagger.mjs", "directory")
pub fn directory(module_source: ModuleSource, path: String) -> Directory

@external(javascript, "../../dagger.mjs", "kind")
pub fn kind(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "moduleName")
pub fn module_name(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "moduleOriginalName")
pub fn module_original_name(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "resolveContextPathFromCaller")
pub fn resolve_context_path_from_caller(
  module_source: ModuleSource,
) -> Promise(String)

@external(javascript, "../../dagger.mjs", "resolveDependency")
pub fn resolve_dependency(
  module_source: ModuleSource,
  dep: ModuleSource,
) -> ModuleSource

@external(javascript, "../../dagger.mjs", "resolveDirectoryFromCaller")
pub fn resolve_directory_from_caller(
  module_source: ModuleSource,
  path: String,
) -> Directory

@external(javascript, "../../dagger.mjs", "resolveFromCaller")
pub fn resolve_from_caller(module_source: ModuleSource) -> ModuleSource

@external(javascript, "../../dagger.mjs", "sourceRootSubpath")
pub fn source_root_subpath(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "sourceDubpath")
pub fn source_dubpath(module_source: ModuleSource) -> Promise(String)

@external(javascript, "../../dagger.mjs", "view")
pub fn view(module_source: ModuleSource, name: String) -> ModuleSourceView

@external(javascript, "../../dagger.mjs", "views")
pub fn views(module_source: ModuleSource) -> Promise(Array(ModuleSourceView))

@external(javascript, "../../dagger.mjs", "withContextDirectory")
pub fn with_context_directory(
  module_source: ModuleSource,
  directory: Directory,
) -> ModuleSource

@external(javascript, "../../dagger.mjs", "withDependencies")
pub fn with_dependencies(
  module_source: ModuleSource,
  dependencies: Array(ModuleDependency),
) -> ModuleSource

@external(javascript, "../../dagger.mjs", "withName")
pub fn with_name(module_source: ModuleSource, name: String) -> ModuleSource

@external(javascript, "../../dagger.mjs", "withSdk")
pub fn with_sdk(module_source: ModuleSource, sdk: String) -> ModuleSource

@external(javascript, "../../dagger.mjs", "withSourceSubpath")
pub fn with_source_subpath(
  module_source: ModuleSource,
  path: String,
) -> ModuleSource

@external(javascript, "../../dagger.mjs", "withView")
pub fn with_view(
  module_source: ModuleSource,
  patterns: Array(String),
) -> ModuleSource
