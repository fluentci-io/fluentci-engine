import gleam/javascript/promise.{type Promise}

pub type Secret

@external(javascript, "../../dagger.mjs", "id")
pub fn id(secret: Secret) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(secret: Secret) -> Promise(String)

@external(javascript, "../../dagger.mjs", "plaintext")
pub fn plaintext(secret: Secret) -> Promise(String)
