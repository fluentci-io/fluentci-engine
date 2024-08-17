module Main where

import Prelude

import Dagger.Class (id)
import Dagger.Client (container, dag)
import Dagger.Container (from, stdout, withExec)
import Effect (Effect)
import Effect.Aff (launchAff_)
import Effect.Class (liftEffect)
import Effect.Class.Console (log)

main :: Effect Unit
main = launchAff_ do
  hello <- liftEffect $ do
    c <- container dag
    c1 <- from c "alpine:latest"
    withExec  c1 ["echo", "Hello from Dagger!"]
  
  stdout hello >>= log
  id hello >>= log
