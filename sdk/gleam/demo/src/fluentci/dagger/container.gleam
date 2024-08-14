import gleam/javascript/array.{type Array}
import gleam/javascript/promise.{type Promise}

pub type Container

@external(javascript, "../../dagger.mjs", "id")
pub fn id(container: Container) -> Promise(String)

@external(javascript, "../../dagger.mjs", "from_")
pub fn from(container: Container, image: String) -> Container

@external(javascript, "../../dagger.mjs", "withExec")
pub fn with_exec(container: Container, args: Array(String)) -> Container

@external(javascript, "../../dagger.mjs", "stdout")
pub fn stdout(container: Container) -> Promise(String)

@external(javascript, "../../dagger.mjs", "stderr")
pub fn stderr(container: Container) -> Promise(String)
