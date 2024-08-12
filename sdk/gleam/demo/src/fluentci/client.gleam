import fluentci/cache.{type Cache}
import fluentci/devbox.{type Devbox}
import fluentci/devenv.{type Devenv}
import fluentci/directory.{type Directory, type File}
import fluentci/flox.{type Flox}
import fluentci/git.{type Git}
import fluentci/mise.{type Mise}
import fluentci/nix.{type Nix}
import fluentci/pipeline.{type Pipeline}
import fluentci/pixi.{type Pixi}
import fluentci/pkgx.{type Pkgx}
import fluentci/secret.{type Secret}
import fluentci/secret_manager.{type SecretManager}

pub type Client

pub type NixArgs {
  NixArgs(impure: Bool)
}

@external(javascript, "../fluentci.mjs", "client")
pub fn dag() -> Client

@external(javascript, "../fluentci.mjs", "azureSecretsManager")
pub fn azure_secrets_manager(
  client: Client,
  region: String,
  access_key_id: String,
  secret_access_key: String,
) -> SecretManager

@external(javascript, "../fluentci.mjs", "azureKeyvault")
pub fn azure_keyvault(
  client: Client,
  client_id: String,
  client_secret: String,
  tenant_id: String,
  keyvault_name: String,
  keyvault_url: String,
) -> SecretManager

@external(javascript, "../fluentci.mjs", "cache")
pub fn cache(client: Client, key: String) -> Cache

@external(javascript, "../fluentci.mjs", "devbox")
pub fn devbox(client: Client) -> Devbox

@external(javascript, "../fluentci.mjs", "devenv")
pub fn devenv(client: Client) -> Devenv

@external(javascript, "../fluentci.mjs", "directory")
pub fn directory(client: Client, path: String) -> Directory

@external(javascript, "../fluentci.mjs", "file")
pub fn file(client: Client, path: String) -> File

@external(javascript, "../fluentci.mjs", "flox")
pub fn flox(client: Client) -> Flox

@external(javascript, "../fluentci.mjs", "git")
pub fn git(client: Client, url: String) -> Git

@external(javascript, "../fluentci.mjs", "googleCloudSecretManager")
pub fn google_cloud_secret_manager(
  client: Client,
  project: String,
  google_credentials_file: String,
) -> SecretManager

@external(javascript, "../fluentci.mjs", "hashicorpVault")
pub fn hashicorp_vault(
  client: Client,
  address: String,
  token: String,
  cacerts: String,
) -> SecretManager

@external(javascript, "../fluentci.mjs", "http")
pub fn http(client: Client, url: String) -> File

@external(javascript, "../fluentci.mjs", "mise")
pub fn mise(client: Client) -> Mise

@external(javascript, "../fluentci.mjs", "nix")
pub fn nix(client: Client, args: NixArgs) -> Nix

@external(javascript, "../fluentci.mjs", "pipeline")
pub fn pipeline(client: Client, name: String) -> Pipeline

@external(javascript, "../fluentci.mjs", "pixi")
pub fn pixi(client: Client) -> Pixi

@external(javascript, "../fluentci.mjs", "pkgx")
pub fn pkgx(client: Client) -> Pkgx

@external(javascript, "../fluentci.mjs", "setSecret")
pub fn set_secret(client: Client, name: String, value: String) -> Secret
