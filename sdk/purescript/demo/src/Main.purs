module Main where

import Prelude

import Effect (Effect)
import Effect.Aff (launchAff_)
import Effect.Class (liftEffect)
import Effect.Class.Console as Console
import FluentCI.Client (dag, pipeline, setSecret)
import FluentCI.Mise (withWorkdir)
import FluentCI.Mise as Mise
import FluentCI.Pipeline (mise, withSecretVariable, withExec, stdout)
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

  m <- liftEffect $ do
    pip <- pipeline dag "mise-demo"
    m1 <- mise pip
    m2 <- withWorkdir m1 "./mise-demo"
    m3 <- Mise.withExec m2 ["mise", "--version"]
    Mise.withExec m3 ["which", "bun"]
  miseOutput <- Mise.stdout m
  
  Console.log value
  Console.log "Secret demo:"
  Console.log output

  Console.log "Mise: "
  Console.log miseOutput
