import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Module

pub type ModuleSource

pub type ModuleDependency

pub type ModuleSourceView

@external(javascript, "../../dagger.mjs", "id")
pub fn id(module: Module) -> Promise(String)

@external(javascript, "../../dagger.mjs", "dependencies")
pub fn dependencies(module: Module) -> Promise(Array(Module))

@external(javascript, "../../dagger.mjs", "dependencyConfig")
pub fn dependency_config(module: Module) -> Promise(Array(ModuleDependency))
