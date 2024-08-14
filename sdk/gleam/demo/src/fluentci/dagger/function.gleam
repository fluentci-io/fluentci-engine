import fluentci/dagger/function_arg.{type FunctionArg}
import fluentci/dagger/type_def.{type TypeDef}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Function

@external(javascript, "../../dagger.mjs", "id")
pub fn id(f: Function) -> Promise(String)

@external(javascript, "../../dagger.mjs", "args")
pub fn args(f: Function) -> Promise(Array(FunctionArg))

@external(javascript, "../../dagger.mjs", "description")
pub fn description(f: Function) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(f: Function) -> Promise(String)

@external(javascript, "../../dagger.mjs", "returnType")
pub fn return_type(f: Function) -> TypeDef

@external(javascript, "../../dagger.mjs", "withArg")
pub fn with_arg(f: Function, arg: FunctionArg) -> Function

@external(javascript, "../../dagger.mjs", "withDescription")
pub fn with_description(f: Function, description: String) -> Function
