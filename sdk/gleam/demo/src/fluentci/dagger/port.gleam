import gleam/javascript/promise.{type Promise}

pub type Port

@external(javascript, "../../dagger.mjs", "id")
pub fn id(port: Port) -> Promise(String)

@external(javascript, "../../dagger.mjs", "description")
pub fn description(port: Port) -> Promise(String)

@external(javascript, "../../dagger.mjs", "experimentalSkipHealthcheck")
pub fn experimental_skip_healthcheck(port: Port) -> Promise(Bool)

@external(javascript, "../../dagger.mjs", "port")
pub fn port(port: Port) -> Promise(Int)

@external(javascript, "../../dagger.mjs", "protocol")
pub fn protocol(port: Port) -> Promise(String)
