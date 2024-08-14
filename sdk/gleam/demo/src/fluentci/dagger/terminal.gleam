import gleam/javascript/promise.{type Promise}

pub type Terminal

@external(javascript, "../../dagger.mjs", "id")
pub fn id(terminal: Terminal) -> Promise(String)
