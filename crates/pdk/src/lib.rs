use extism_pdk::*;
use fluentci_types::{nix::NixArgs, secret::*, Module};
use proto::Proto;
use secret::Secret;
use secret_manager::SecretManager;

use self::{
    cache::Cache, devbox::Devbox, devenv::Devenv, directory::Directory, envhub::Envhub, file::File,
    flox::Flox, git::Git, hermit::Hermit, mise::Mise, nix::Nix, pipeline::Pipeline, pixi::Pixi,
    pkgx::Pkgx,
};

pub mod cache;
pub mod devbox;
pub mod devenv;
pub mod directory;
pub mod envhub;
pub mod file;
pub mod flox;
pub mod git;
pub mod hermit;
pub mod mise;
pub mod nix;
pub mod pipeline;
pub mod pixi;
pub mod pkgx;
pub mod proto;
pub mod secret;
pub mod secret_manager;

#[host_fn]
extern "ExtismHost" {
    fn cache(key: String) -> Json<Cache>;
    fn devbox() -> Json<Devbox>;
    fn devenv() -> Json<Devenv>;
    fn directory(path: String) -> Json<Directory>;
    fn envhub() -> Json<Envhub>;
    fn file(path: String) -> Json<File>;
    fn flox() -> Json<Flox>;
    fn git(url: String) -> Json<Git>;
    fn http(url: String) -> Json<File>;
    fn hermit() -> Json<Hermit>;
    fn mise() -> Json<Mise>;
    fn nix(args: Json<NixArgs>) -> Json<Nix>;
    fn pipeline(name: String) -> Json<Pipeline>;
    fn pixi() -> Json<Pixi>;
    fn proto() -> Json<Proto>;
    fn pkgx() -> Json<Pkgx>;
    fn get_env(key: String) -> String;
    fn has_env(key: String) -> Json<bool>;
    fn set_envs(envs: Json<Vec<(String, String)>>);
    fn remove_env(key: String);
    fn get_os() -> String;
    fn get_arch() -> String;
    fn call(opts: Json<Module>) -> String;
    fn add_secretmanager(provider: Json<Provider>) -> String;
    fn set_secret(params: Json<Vec<String>>) -> String;
}

pub struct Client {}

pub fn dag() -> Client {
    Client {}
}

impl Client {
    pub fn cache(&self, key: &str) -> Result<Cache, Error> {
        unsafe { cache(key.into()) }.map(|cache| cache.into_inner())
    }

    pub fn devbox(&self) -> Result<Devbox, Error> {
        unsafe { devbox() }.map(|devbox| devbox.into_inner())
    }

    pub fn devenv(&self) -> Result<Devenv, Error> {
        unsafe { devenv() }.map(|devenv| devenv.into_inner())
    }

    pub fn directory(&self, path: &str) -> Result<Directory, Error> {
        unsafe { directory(path.to_string()) }.map(|directory| directory.into_inner())
    }

    pub fn envhub(&self) -> Result<Envhub, Error> {
        unsafe { envhub() }.map(|envhub| envhub.into_inner())
    }

    pub fn file(&self, path: &str) -> Result<File, Error> {
        unsafe { file(path.into()) }.map(|file| file.into_inner())
    }

    pub fn flox(&self) -> Result<Flox, Error> {
        unsafe { flox() }.map(|flox| flox.into_inner())
    }

    pub fn git(&self, url: &str) -> Result<Git, Error> {
        unsafe { git(url.into()) }.map(|git| git.into_inner())
    }

    pub fn http(&self, url: &str) -> Result<File, Error> {
        unsafe { http(url.into()) }.map(|file| file.into_inner())
    }

    pub fn mise(&self) -> Result<Mise, Error> {
        unsafe { mise() }.map(|mise| mise.into_inner())
    }

    pub fn hermit(&self) -> Result<Hermit, Error> {
        unsafe { hermit() }.map(|hermit| hermit.into_inner())
    }

    pub fn nix(&self, args: NixArgs) -> Result<Nix, Error> {
        unsafe { nix(Json(args)) }.map(|nix| nix.into_inner())
    }

    pub fn pipeline(&self, name: &str) -> Result<Pipeline, Error> {
        unsafe { pipeline(name.into()) }.map(|pipeline| pipeline.into_inner())
    }

    pub fn pixi(&self) -> Result<Pixi, Error> {
        unsafe { pixi() }.map(|pixi| pixi.into_inner())
    }

    pub fn pkgx(&self) -> Result<Pkgx, Error> {
        unsafe { pkgx() }.map(|pkgx| pkgx.into_inner())
    }

    pub fn proto(&self) -> Result<Proto, Error> {
        unsafe { proto() }.map(|proto| proto.into_inner())
    }

    pub fn get_env(&self, key: &str) -> Result<String, Error> {
        unsafe { get_env(key.into()) }
    }

    pub fn has_env(&self, key: &str) -> Result<bool, Error> {
        unsafe { has_env(key.into()) }.map(|has_env| has_env.into_inner())
    }

    pub fn set_envs(&self, envs: Vec<(String, String)>) -> Result<(), Error> {
        unsafe { set_envs(envs.into()) }
    }

    pub fn remove_env(&self, key: &str) -> Result<(), Error> {
        unsafe { remove_env(key.into()) }
    }

    pub fn get_os(&self) -> Result<String, Error> {
        unsafe { get_os() }
    }

    pub fn get_arch(&self) -> Result<String, Error> {
        unsafe { get_arch() }
    }

    pub fn call(&self, url: &str, func: &str, args: Vec<&str>) -> Result<String, Error> {
        unsafe {
            call(Json(Module {
                url: url.into(),
                function: func.into(),
                args: args.join(" "),
            }))
        }
    }

    pub fn google_secret_manager(
        &self,
        project: &str,
        google_credentials_file: &str,
    ) -> Result<SecretManager, Error> {
        let provider = Provider::Google(GoogleConfig {
            google_project: Some(project.to_string()),
            google_credentials_file: Some(google_credentials_file.into()),
            ..Default::default()
        });
        let id = unsafe { add_secretmanager(Json(provider))? };
        Ok(SecretManager { id })
    }

    pub fn aws_secrets_manager(
        &self,
        region: &str,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Result<SecretManager, Error> {
        let provider = Provider::Aws(AwsConfig {
            aws_region: region.into(),
            aws_access_key_id: Some(access_key_id.to_string()),
            aws_secret_access_key: Some(secret_access_key.to_string()),
        });
        let id = unsafe { add_secretmanager(Json(provider))? };
        Ok(SecretManager { id })
    }

    pub fn azure_keyvault(
        &self,
        client_id: &str,
        client_secret: &str,
        tenant_id: &str,
        keyvault_name: &str,
        keyvault_url: &str,
    ) -> Result<SecretManager, Error> {
        let provider = Provider::Azure(AzureConfig {
            credential: AzureCredential {
                azure_client_id: Some(client_id.to_string()),
                azure_client_secret: Some(client_secret.to_string()),
                azure_tenant_id: Some(tenant_id.to_string()),
            },
            azure_keyvault_name: Some(keyvault_name.to_string()),
            azure_keyvault_url: Some(keyvault_url.to_string()),
        });
        let id = unsafe { add_secretmanager(Json(provider))? };
        Ok(SecretManager { id })
    }

    pub fn hashicorp_vault(
        &self,
        address: &str,
        token: &str,
        cacert: Option<&str>,
    ) -> Result<SecretManager, Error> {
        let provider = Provider::Hashicorp(HashicorpVaultConfig {
            vault_address: Some(address.to_string()),
            vault_token: Some(token.to_string()),
            vault_cacert: cacert.map(|x| x.into()),
        });
        let id = unsafe { add_secretmanager(Json(provider))? };
        Ok(SecretManager { id })
    }

    pub fn set_secret(&self, name: &str, value: &str) -> Result<Secret, Error> {
        let id = unsafe { set_secret(Json(vec![name.into(), value.into()]))? };
        Ok(Secret {
            id,
            name: name.into(),
            mount: "default".into(),
        })
    }
}
