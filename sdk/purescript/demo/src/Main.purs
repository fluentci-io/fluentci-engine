module Main where

import Prelude

import Effect (Effect)
import Effect.Aff (launchAff_)
import Effect.Class (liftEffect)
import Effect.Class.Console as Console
import FluentCI.Client (dag, pipeline, setSecret)
import FluentCI.Pipeline (withSecretVariable, withExec, stdout)
import FluentCI.Secret (plaintext)

main :: Effect Unit
main = launchAff_ do
  secret <- liftEffect $ setSecret dag "GITHUB" "my-github-token"
  value <- plaintext secret
  p <- liftEffect $ do
    p1 <- pipeline dag "secret-demo"
    p2 <- withSecretVariable p1 "GITHUB" secret
    withExec p2 ["echo", "$GITHUB"]
  output <- stdout p
  Console.log value
  Console.log "Secret demo:"
  Console.log output
