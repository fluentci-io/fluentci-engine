import fluentci/dagger/module.{type ModuleSourceView}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

@external(javascript, "../../dagger.mjs", "id")
pub fn id(module_source_view: ModuleSourceView) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(module_source_view: ModuleSourceView) -> Promise(String)

@external(javascript, "../../dagger.mjs", "patterns")
pub fn patterns(module_source_view: ModuleSourceView) -> Promise(Array(String))
