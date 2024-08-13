module Main where

import Prelude

import Effect (Effect)
import Effect.Aff (launchAff_)
import Effect.Class (liftEffect)
import Effect.Class.Console as Console
import FluentCI.Client (dag, setSecret)
import FluentCI.Secret (plaintext)

main :: Effect Unit
main = launchAff_ do
  secret <- liftEffect $ setSecret dag "secret" "value 123"
  result <- plaintext secret
  Console.log result
