module FluentCI.Client where

import Effect (Effect)
import FluentCI.Cache (Cache)
import FluentCI.Secret (Secret)
import FluentCI.SecretManager (SecretManager)

data Client

foreign import dag :: Client

foreign import setSecret :: Client -> String -> String -> Effect Secret

foreign import awsSecretsManager :: Client -> String -> String -> String -> String -> Effect SecretManager

foreign import azureKeyvault :: Client -> String -> String -> String -> String -> String -> Effect SecretManager

foreign import cache :: Client -> String -> Effect Cache