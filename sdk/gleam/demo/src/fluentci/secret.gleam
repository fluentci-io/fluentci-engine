import gleam/javascript/promise.{type Promise}

pub type Secret

@external(javascript, "../fluentci.mjs", "plaintext")
pub fn plaintext_(secret: Secret) -> Promise(String)

pub fn plaintext(secret: Secret) -> Promise(String) {
  plaintext_(secret)
}
