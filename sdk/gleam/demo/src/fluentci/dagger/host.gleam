import fluentci/dagger/directory.{type Directory}
import fluentci/dagger/file.{type File}
import fluentci/dagger/secret.{type Secret}
import fluentci/dagger/service.{type Service}
import fluentci/dagger/socket.{type Socket}
import gleam/javascript/promise.{type Promise}

pub type Host

@external(javascript, "../../dagger.mjs", "id")
pub fn id(host: Host) -> Promise(String)

@external(javascript, "../../dagger.mjs", "directory")
pub fn directory(host: Host, path: String) -> Directory

@external(javascript, "../../dagger.mjs", "file")
pub fn file(host: Host, path: String) -> File

@external(javascript, "../../dagger.mjs", "service")
pub fn service(host: Host) -> Service

@external(javascript, "../../dagger.mjs", "setSecretFile")
pub fn set_secret_file(host: Host, name: String, path: String) -> Secret

@external(javascript, "../../dagger.mjs", "tunnel")
pub fn tunnel(host: Host, service: Service) -> Service

@external(javascript, "../../dagger.mjs", "unixSocket")
pub fn unix_socket(host: Host, path: String) -> Socket
