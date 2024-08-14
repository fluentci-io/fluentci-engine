import fluentci/dagger/function_arg.{type JSON}
import gleam/javascript/promise.{type Promise}

pub type FunctionCallArgValue

@external(javascript, "../../dagger.mjs", "id")
pub fn id(f: FunctionCallArgValue) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(f: FunctionCallArgValue) -> Promise(String)

@external(javascript, "../../dagger.mjs", "value")
pub fn value(f: FunctionCallArgValue) -> Promise(JSON)
