import gleam/javascript/promise.{type Promise}

pub type Service

@external(javascript, "../fluentci.mjs", "id")
pub fn id(service: Service) -> Promise(String)
