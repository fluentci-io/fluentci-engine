import gleam/javascript/promise.{type Promise}

pub type JSON

pub type FunctionArg

@external(javascript, "../../dagger.mjs", "id")
pub fn id(f: FunctionArg) -> Promise(String)

@external(javascript, "../../dagger.mjs", "defaultValue")
pub fn default_value(f: FunctionArg) -> Promise(JSON)

@external(javascript, "../../dagger.mjs", "description")
pub fn description(f: FunctionArg) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(f: FunctionArg) -> Promise(String)

@external(javascript, "../../dagger.mjs", "typeDef")
pub fn type_def(f: FunctionArg) -> Promise(JSON)
