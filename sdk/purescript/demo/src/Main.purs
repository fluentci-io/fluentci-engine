module Main where

import Prelude
import Effect (Effect)
import Effect.Aff (launchAff_)
import Effect.Class (liftEffect)
import Effect.Class.Console as Console
import FluentCI.Client (dag, pipeline)
import FluentCI.Pipeline (Pipeline, stdout, withEnvVariable, withExec, withWorkdir)

build :: Effect Pipeline
build =
  pipeline "demo" dag
    >>= withWorkdir "."
    >>= withEnvVariable "MESSAGE" "Hello from PureScript!"
    >>= withExec [ "printf", "%s", "$MESSAGE" ]

main :: Effect Unit
main = launchAff_ do
  built <- liftEffect build
  built # stdout >>= Console.log
