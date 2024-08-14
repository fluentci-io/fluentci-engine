import fluentci/dagger/enum_value_type_def.{type EnumValueTypeDef}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type EnumTypeDef

@external(javascript, "../../dagger.mjs", "id")
pub fn id(f: EnumTypeDef) -> Promise(String)

@external(javascript, "../../dagger.mjs", "description")
pub fn description(f: EnumTypeDef) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(f: EnumTypeDef) -> Promise(String)

@external(javascript, "../../dagger.mjs", "sourceModuleName")
pub fn source_module_name(f: EnumTypeDef) -> Promise(String)

@external(javascript, "../../dagger.mjs", "values")
pub fn values(f: EnumTypeDef) -> Promise(Array(EnumValueTypeDef))
