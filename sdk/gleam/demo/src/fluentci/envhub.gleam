import fluentci/cache.{type Cache}
import fluentci/secret.{type Secret}
import fluentci/service.{type Service}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Envhub

@external(javascript, "../fluentci.mjs", "asService")
pub fn as_service(envhub: Envhub) -> Service

@external(javascript, "../fluentci.mjs", "id")
pub fn id(envhub: Envhub) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stderr")
pub fn stderr(envhub: Envhub) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stdout")
pub fn stdout(envhub: Envhub) -> Promise(String)

@external(javascript, "../fluentci.mjs", "use")
pub fn use_environment(envhub: Envhub, environment: String) -> Envhub

@external(javascript, "../fluentci.mjs", "waitOn")
pub fn wait_on(envhub: Envhub, port: int, timeout: int) -> Envhub

@external(javascript, "../fluentci.mjs", "withCache")
pub fn with_cache(envhub: Envhub, cache: Cache) -> Envhub

@external(javascript, "../fluentci.mjs", "withEnvVariable")
pub fn with_env_variable(envhub: Envhub, name: String, value: String) -> Envhub

@external(javascript, "../fluentci.mjs", "withExec")
pub fn with_exec(envhub: Envhub, command: Array(String)) -> Envhub

@external(javascript, "../fluentci.mjs", "withSecretVariable")
pub fn with_secret_variable(
  envhub: Envhub,
  name: String,
  secret: Secret,
) -> Envhub

@external(javascript, "../fluentci.mjs", "withService")
pub fn with_service(envhub: Envhub, service: Service) -> Envhub

@external(javascript, "../fluentci.mjs", "withWorkdir")
pub fn with_workdir(envhub: Envhub, path: String) -> Envhub
