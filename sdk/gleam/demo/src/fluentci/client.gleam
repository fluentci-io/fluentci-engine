import fluentci/cache.{type Cache}
import fluentci/devbox.{type Devbox}
import fluentci/devenv.{type Devenv}
import fluentci/directory.{type Directory}
import fluentci/file.{type File}
import fluentci/flox.{type Flox}
import fluentci/git.{type Git}
import fluentci/nix.{type Nix}
import fluentci/pipeline.{type Pipeline}
import fluentci/pkgx.{type Pkgx}
import fluentci/secret.{type Secret}
import fluentci/secret_manager.{type SecretManager}

pub type Client

@external(javascript, "../fluentci.mjs", "client")
pub fn dag() -> Client

@external(javascript, "../fluentci.mjs", "setSecret")
pub fn set_secret_(client: Client, name: String, value: String) -> Secret

pub fn azure_secrets_manager(
  client: Client,
  region: String,
  access_key_id: String,
  secret_access_key: String,
) -> SecretManager {
  todo
}

pub fn azure_keyvault(
  client: Client,
  client_id: String,
  client_secret: String,
  tenant_id: String,
  keyvault_name: String,
  keyvault_url: String,
) -> SecretManager {
  todo
}

pub fn cache(client: Client, key: String) -> Cache {
  todo
}

pub fn devbox(client: Client) -> Devbox {
  todo
}

pub fn devenv(client: Client) -> Devenv {
  todo
}

pub fn directory(client: Client, path: String) -> Directory {
  todo
}

pub fn file(client: Client, path: String) -> File {
  todo
}

pub fn flox(client: Client) -> Flox {
  todo
}

pub fn git(client: Client, url: String) -> Git {
  todo
}

pub fn google_cloud_secret_manager(
  client: Client,
  project: String,
  google_credentials_file: String,
) -> SecretManager {
  todo
}

pub fn hashicorp_vault(
  client: Client,
  address: String,
  token: String,
  cacerts: String,
) -> SecretManager {
  todo
}

pub fn http(client: Client, url: String) -> File {
  todo
}

pub fn nix(client: Client) -> Nix {
  todo
}

pub fn pipeline(client: Client, name: String) -> Pipeline {
  todo
}

pub fn pkgx(client: Client) -> Pkgx {
  todo
}

pub fn set_secret(client: Client, name: String, value: String) -> Secret {
  set_secret_(client, name, value)
}
