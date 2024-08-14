import fluentci/dagger/module.{type ModuleDependency, type ModuleSource}
import gleam/javascript/promise.{type Promise}

@external(javascript, "../../dagger.mjs", "id")
pub fn id(module_dependency: ModuleDependency) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(module_dependency: ModuleDependency) -> Promise(String)

@external(javascript, "../../dagger.mjs", "source")
pub fn source(module_dependency: ModuleDependency) -> ModuleSource
