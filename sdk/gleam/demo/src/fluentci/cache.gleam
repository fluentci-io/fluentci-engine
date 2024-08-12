import gleam/javascript/promise.{type Promise}

pub type Cache

@external(javascript, "../fluentci.mjs", "id")
pub fn id_(cache: Cache) -> Promise(String)

@external(javascript, "../fluentci.mjs", "key")
pub fn key_(cache: Cache) -> Promise(String)

pub fn id(cache: Cache) -> Promise(String) {
  id_(cache)
}

pub fn key(cache: Cache) -> Promise(String) {
  key_(cache)
}
