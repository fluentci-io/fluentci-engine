import fluentci/directory.{type Directory}
import gleam/javascript/promise.{type Promise}

pub type Git

@external(javascript, "../fluentci.mjs", "git")
pub fn branch(g: Git, name: String) -> Git

@external(javascript, "../fluentci.mjs", "commit")
pub fn commit(g: Git) -> Git

@external(javascript, "../fluentci.mjs", "id")
pub fn id(g: Git) -> Promise(String)

@external(javascript, "../fluentci.mjs", "tree")
pub fn tree(g: Git) -> Directory
