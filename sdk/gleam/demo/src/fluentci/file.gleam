import fluentci/directory.{type Directory, type File}
import gleam/javascript/promise.{type Promise}

@external(javascript, "../fluentci.mjs", "id")
pub fn id(f: File) -> Promise(String)

@external(javascript, "../fluentci.mjs", "chmod")
pub fn chmod(f: File, mode: int) -> Promise(File)

@external(javascript, "../fluentci.mjs", "md5")
pub fn md5(f: File) -> Promise(String)

@external(javascript, "../fluentci.mjs", "path")
pub fn path(f: File) -> Promise(String)

@external(javascript, "../fluentci.mjs", "sha256")
pub fn sha256(f: File) -> Promise(String)

@external(javascript, "../fluentci.mjs", "tarCzvf")
pub fn tar_czvf(f: File, output_dir: String) -> Directory

@external(javascript, "../fluentci.mjs", "unzip")
pub fn unzip(f: File, output_dir: String) -> Directory

@external(javascript, "../fluentci.mjs", "zip")
pub fn zip(f: File) -> Directory
