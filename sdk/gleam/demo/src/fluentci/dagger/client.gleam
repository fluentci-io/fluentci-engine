import fluentci/dagger/cache_volume.{type CacheVolume}
import fluentci/dagger/container.{type Container}
import fluentci/dagger/directory.{type Directory}
import fluentci/dagger/file.{type File}
import fluentci/dagger/host.{type Host}

pub type Client

@external(javascript, "../../dagger.mjs", "client")
pub fn dag() -> Client

@external(javascript, "../../dagger.mjs", "pipeline")
pub fn pipeline(client: Client, name: String) -> Client

@external(javascript, "../../dagger.mjs", "container")
pub fn container(client: Client) -> Container

@external(javascript, "../../dagger.mjs", "cacheVolume")
pub fn cache_volume(client: Client, key: String) -> CacheVolume

@external(javascript, "../../dagger.mjs", "emptyDirectory")
pub fn directory(client: Client) -> Directory

@external(javascript, "../../dagger.mjs", "git")
pub fn git(client: Client, url: String) -> Directory

@external(javascript, "../../dagger.mjs", "host")
pub fn host(client: Client) -> Host

@external(javascript, "../../dagger.mjs", "http")
pub fn http(client: Client, url: String) -> File
