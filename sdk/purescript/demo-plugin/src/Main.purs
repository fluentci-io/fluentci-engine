module Main where

import Prelude
import Data.Either (Either(..))
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Plugin as P

foreign import input :: Effect String
foreign import output :: String -> Effect Unit

greet :: Effect Unit
greet = do
  name <- input
  result <- P.pipeline "purescript-greeting"
    # P.exec ["/usr/bin/printf", "%s", name]
    # P.stdout
  case result of
    Left message -> throw message
    Right value -> output value
