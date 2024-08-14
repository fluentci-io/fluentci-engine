import gleam/javascript/promise.{type Promise}

pub type File

@external(javascript, "../../dagger.mjs", "id")
pub fn id(file: File) -> Promise(String)

@external(javascript, "../../dagger.mjs", "contents")
pub fn contents(file: File) -> Promise(String)

@external(javascript, "../../dagger.mjs", "digest")
pub fn digest(file: File) -> Promise(String)

@external(javascript, "../../dagger.mjs", "export_")
pub fn export(file: File, path: String) -> Promise(String)

@external(javascript, "../../dagger.mjs", "name")
pub fn name(file: File) -> Promise(String)

@external(javascript, "../../dagger.mjs", "size")
pub fn size(file: File) -> Promise(Int)

@external(javascript, "../../dagger.mjs", "sync")
pub fn sync(file: File) -> Promise(File)

@external(javascript, "../../dagger.mjs", "withName")
pub fn with_name(file: File, name: String) -> File

@external(javascript, "../../dagger.mjs", "withTimestamps")
pub fn with_timestamps(file: File, timestamp: Int) -> File
