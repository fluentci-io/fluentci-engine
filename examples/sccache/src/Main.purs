module Main where

import Prelude
import Data.Argonaut.Decode (decodeJson)
import Data.Argonaut.Parser (jsonParser)
import Data.Either (Either(..))
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Plugin as P
import Sccache as Sccache

build :: Effect Unit
build = do
  input <- P.input >>= Sccache.require
  json <- case jsonParser input of
    Left error -> throw error
    Right value -> pure value
  arguments <- case decodeJson json of
    Left error -> throw (show error)
    Right value -> pure value
  Sccache.buildProject arguments >>= P.output >>= Sccache.require
