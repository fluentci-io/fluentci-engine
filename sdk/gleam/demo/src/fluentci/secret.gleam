import gleam/javascript/promise.{type Promise}

pub type Secret

@external(javascript, "../fluentci.mjs", "id")
pub fn id(secret: Secret) -> Promise(String)

@external(javascript, "../fluentci.mjs", "mount")
pub fn mount(secret: Secret) -> Promise(String)

@external(javascript, "../fluentci.mjs", "path")
pub fn name(secret: Secret) -> Promise(String)

@external(javascript, "../fluentci.mjs", "plaintext")
pub fn plaintext(secret: Secret) -> Promise(String)
