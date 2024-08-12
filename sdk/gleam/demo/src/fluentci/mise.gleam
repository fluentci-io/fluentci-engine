import fluentci/cache.{type Cache}
import fluentci/secret.{type Secret}
import fluentci/service.{type Service}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Mise

@external(javascript, "../fluentci.mjs", "asService")
pub fn as_service(mise: Mise) -> Service

@external(javascript, "../fluentci.mjs", "id")
pub fn id(mise: Mise) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stderr")
pub fn stderr(mise: Mise) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stdout")
pub fn stdout(mise: Mise) -> Promise(String)

@external(javascript, "../fluentci.mjs", "waitOn")
pub fn wait_on(mise: Mise, port: int, timeout: int) -> Mise

@external(javascript, "../fluentci.mjs", "withCache")
pub fn with_cache(mise: Mise, cache: Cache) -> Mise

@external(javascript, "../fluentci.mjs", "withEnvVariable")
pub fn with_env_variable(mise: Mise, name: String, value: String) -> Mise

@external(javascript, "../fluentci.mjs", "withExec")
pub fn with_exec(mise: Mise, command: Array(String)) -> Mise

@external(javascript, "../fluentci.mjs", "withSecretVariable")
pub fn with_secret_variable(mise: Mise, name: String, secret: Secret) -> Mise

@external(javascript, "../fluentci.mjs", "withService")
pub fn with_service(mise: Mise, service: Service) -> Mise

@external(javascript, "../fluentci.mjs", "withWorkdir")
pub fn with_workdir(mise: Mise, path: String) -> Mise
