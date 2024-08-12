import fluentci/cache.{type Cache}
import fluentci/devbox.{type Devbox}
import fluentci/devenv.{type Devenv}
import fluentci/flox.{type Flox}
import fluentci/mise.{type Mise}
import fluentci/nix.{type Nix}
import fluentci/pixi.{type Pixi}
import fluentci/pkgx.{type Pkgx}
import fluentci/secret.{type Secret}
import fluentci/service.{type Service}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Directory

@external(javascript, "../fluentci.mjs", "devbox")
pub fn devbox(d: Directory) -> Devbox

@external(javascript, "../fluentci.mjs", "devenv")
pub fn devenv(d: Directory) -> Devenv

@external(javascript, "../fluentci.mjs", "entries")
pub fn entries(d: Directory) -> Promise(Array(String))

@external(javascript, "../fluentci.mjs", "flox")
pub fn flox(d: Directory) -> Flox

@external(javascript, "../fluentci.mjs", "mise")
pub fn mise(d: Directory) -> Mise

@external(javascript, "../fluentci.mjs", "nix")
pub fn nix(d: Directory) -> Nix

@external(javascript, "../fluentci.mjs", "pixi")
pub fn pixi(d: Directory) -> Pixi

@external(javascript, "../fluentci.mjs", "pkgx")
pub fn pkgx(d: Directory) -> Pkgx

@external(javascript, "../fluentci.mjs", "directory")
pub fn directory(d: Directory, path: String) -> Directory

@external(javascript, "../fluentci.mjs", "stderr")
pub fn stderr(d: Directory) -> Promise(String)

@external(javascript, "../fluentci.mjs", "stdout")
pub fn stdout(d: Directory) -> Promise(String)

@external(javascript, "../fluentci.mjs", "waitOn")
pub fn wait_on(d: Directory, port: int, timeout: int) -> Directory

@external(javascript, "../fluentci.mjs", "withCache")
pub fn with_cache(d: Directory, cache: Cache) -> Directory

@external(javascript, "../fluentci.mjs", "withEnvVariable")
pub fn with_env_variable(d: Directory, name: String, value: String) -> Directory

@external(javascript, "../fluentci.mjs", "withExec")
pub fn with_exec(d: Directory, command: Array(String)) -> Directory

@external(javascript, "../fluentci.mjs", "withSecretVariable")
pub fn with_secret_variable(
  d: Directory,
  name: String,
  secret: Secret,
) -> Directory

@external(javascript, "../fluentci.mjs", "withService")
pub fn with_service(d: Directory, service: Service) -> Directory

@external(javascript, "../fluentci.mjs", "withWorkdir")
pub fn with_workdir(d: Directory, path: String) -> Directory

@external(javascript, "../fluentci.mjs", "withFile")
pub fn with_file(d: Directory, file_id: String) -> Directory

@external(javascript, "../fluentci.mjs", "zip")
pub fn zip(d: Directory) -> File

@external(javascript, "../fluentci.mjs", "tarCzvf")
pub fn tar_czvf(d: Directory) -> File

pub type File
