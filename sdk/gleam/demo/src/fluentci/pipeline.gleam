import fluentci/cache.{type Cache}
import fluentci/devbox.{type Devbox}
import fluentci/devenv.{type Devenv}
import fluentci/envhub.{type Envhub}
import fluentci/mise.{type Mise}
import fluentci/nix.{type Nix}
import fluentci/pixi.{type Pixi}
import fluentci/pkgx.{type Pkgx}
import fluentci/secret.{type Secret}
import fluentci/service.{type Service}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Pipeline

@external(javascript, "../fluentci.mjs", "asService")
pub fn as_service(pipeline: Pipeline) -> Service

@external(javascript, "../fluentci.mjs", "id")
pub fn id(pipeline: Pipeline) -> Promise(String)

@external(javascript, "../fluentci.mjs", "devbox")
pub fn devbox(pipeline: Pipeline) -> Devbox

@external(javascript, "../fluentci.mjs", "devenv")
pub fn devenv(pipeline: Pipeline) -> Devenv

@external(javascript, "../fluentci.mjs", "envhub")
pub fn envhub(pipeline: Pipeline) -> Envhub

@external(javascript, "../fluentci.mjs", "mise")
pub fn mise(pipeline: Pipeline) -> Mise

@external(javascript, "../fluentci.mjs", "nix")
pub fn nix(pipeline: Pipeline) -> Nix

@external(javascript, "../fluentci.mjs", "pixi")
pub fn pixi(pipeline: Pipeline) -> Pixi

@external(javascript, "../fluentci.mjs", "pkgx")
pub fn pkgx(pipeline: Pipeline) -> Pkgx

@external(javascript, "../fluentci.mjs", "stderr")
pub fn stderr(pipeline: Pipeline) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stdout")
pub fn stdout(pipeline: Pipeline) -> Promise(String)

@external(javascript, "../fluentci.mjs", "waitOn")
pub fn wait_on(pipeline: Pipeline, port: int, timeout: int) -> Pipeline

@external(javascript, "../fluentci.mjs", "withCache")
pub fn with_cache(pipeline: Pipeline, cache: Cache) -> Pipeline

@external(javascript, "../fluentci.mjs", "withEnvVariable")
pub fn with_env_variable(
  pipeline: Pipeline,
  name: String,
  value: String,
) -> Pipeline

@external(javascript, "../fluentci.mjs", "withExec")
pub fn with_exec(pipeline: Pipeline, command: Array(String)) -> Pipeline

@external(javascript, "../fluentci.mjs", "withSecretVariable")
pub fn with_secret_variable(
  pipeline: Pipeline,
  name: String,
  secret: Secret,
) -> Pipeline

@external(javascript, "../fluentci.mjs", "withService")
pub fn with_service(pipeline: Pipeline, service: Service) -> Pipeline

@external(javascript, "../fluentci.mjs", "withWorkdir")
pub fn with_workdir(pipeline: Pipeline, path: String) -> Pipeline
