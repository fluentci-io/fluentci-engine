import fluentci/cache.{type Cache}
import fluentci/secret.{type Secret}
import fluentci/service.{type Service}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Pixi

@external(javascript, "../fluentci.mjs", "asService")
pub fn as_service(pixi: Pixi) -> Service

@external(javascript, "../fluentci.mjs", "id")
pub fn id(pixi: Pixi) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stderr")
pub fn stderr(pixi: Pixi) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stdout")
pub fn stdout(pixi: Pixi) -> Promise(String)

@external(javascript, "../fluentci.mjs", "waitOn")
pub fn wait_on(pixi: Pixi, port: int, timeout: int) -> Pixi

@external(javascript, "../fluentci.mjs", "withCache")
pub fn with_cache(pixi: Pixi, cache: Cache) -> Pixi

@external(javascript, "../fluentci.mjs", "withEnvVariable")
pub fn with_env_variable(pixi: Pixi, name: String, value: String) -> Pixi

@external(javascript, "../fluentci.mjs", "withExec")
pub fn with_exec(pixi: Pixi, command: Array(String)) -> Pixi

@external(javascript, "../fluentci.mjs", "withSecretVariable")
pub fn with_secret_variable(pixi: Pixi, name: String, secret: Secret) -> Pixi

@external(javascript, "../fluentci.mjs", "withService")
pub fn with_service(pixi: Pixi, service: Service) -> Pixi

@external(javascript, "../fluentci.mjs", "withWorkdir")
pub fn with_workdir(pixi: Pixi, path: String) -> Pixi
