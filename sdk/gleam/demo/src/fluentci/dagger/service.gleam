import fluentci/dagger/port.{type Port}
import fluentci/dagger/void.{type Void}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Service

@external(javascript, "../../dagger.mjs", "id")
pub fn id(service: Service) -> Promise(String)

@external(javascript, "../../dagger.mjs", "endpoint")
pub fn endpoint(service: Service) -> Promise(String)

@external(javascript, "../../dagger.mjs", "hostname")
pub fn hostname(service: Service) -> Promise(String)

@external(javascript, "../../dagger.mjs", "ports")
pub fn ports(service: Service) -> Promise(Array(Port))

@external(javascript, "../../dagger.mjs", "start")
pub fn start(service: Service) -> Promise(Service)

@external(javascript, "../../dagger.mjs", "stop")
pub fn stop(service: Service) -> Promise(Service)

@external(javascript, "../../dagger.mjs", "up")
pub fn up(service: Service) -> Promise(Void)
