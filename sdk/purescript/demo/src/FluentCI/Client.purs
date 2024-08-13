module FluentCI.Client where

import Effect (Effect)
import FluentCI.Cache (Cache)
import FluentCI.Devbox (Devbox)
import FluentCI.Devenv (Devenv)
import FluentCI.Directory (Directory, File)
import FluentCI.Flox (Flox)
import FluentCI.Git (Git)
import FluentCI.Mise (Mise)
import FluentCI.Nix (Nix, NixArgs)
import FluentCI.Pipeline (Pipeline)
import FluentCI.Pixi (Pixi)
import FluentCI.Pkgx (Pkgx)
import FluentCI.Secret (Secret)
import FluentCI.SecretManager (SecretManager)

data Client

foreign import dag :: Client

foreign import awsSecretsManager :: Client -> String -> String -> String -> String -> Effect SecretManager

foreign import azureKeyvault :: Client -> String -> String -> String -> String -> String -> Effect SecretManager

foreign import cache :: Client -> String -> Effect Cache

foreign import devbox :: Client -> Effect Devbox

foreign import devenv :: Client -> Effect Devenv

foreign import directory :: Client -> String -> Effect Directory

foreign import file :: Client -> String -> Effect File

foreign import flox :: Client -> Effect Flox

foreign import git :: Client -> String -> Effect Git

foreign import googleCloudSecretManager :: Client -> String -> String -> Effect SecretManager

foreign import hashicorpVault :: Client -> String -> String -> String -> String -> Effect SecretManager

foreign import http :: Client -> String -> Effect File

foreign import mise :: Client -> Effect Mise

foreign import nix :: Client -> NixArgs -> Effect Nix

foreign import pipeline :: Client -> String -> Effect Pipeline

foreign import pixi :: Client -> Effect Pixi


foreign import pkgx :: Client -> Effect Pkgx

foreign import setSecret :: Client -> String -> String -> Effect Secret
