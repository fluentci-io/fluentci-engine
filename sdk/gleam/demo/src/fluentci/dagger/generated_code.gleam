import fluentci/dagger/directory.{type Directory}
import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type GeneratedCode

@external(javascript, "../../dagger.mjs", "id")
pub fn id(f: GeneratedCode) -> Promise(String)

@external(javascript, "../../dagger.mjs", "code")
pub fn code(f: GeneratedCode) -> Directory

@external(javascript, "../../dagger.mjs", "vcsGeneratedPaths")
pub fn vcs_generated_paths(f: GeneratedCode) -> Promise(Array(String))

@external(javascript, "../../dagger.mjs", "vcsIgnoredPaths")
pub fn vcs_ignored_paths(f: GeneratedCode) -> Promise(Array(String))

@external(javascript, "../../dagger.mjs", "withVSCGeneratedPaths")
pub fn with_vsc_generated_paths(
  f: GeneratedCode,
  paths: Array(String),
) -> GeneratedCode

@external(javascript, "../../dagger.mjs", "withVSCIgnoredPaths")
pub fn with_vsc_ignored_paths(
  f: GeneratedCode,
  paths: Array(String),
) -> GeneratedCode
