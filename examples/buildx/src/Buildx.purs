module Buildx where

import Prelude
import Data.Argonaut.Core (Json)
import Data.Argonaut.Decode (class DecodeJson, decodeJson, getFieldOptional)
import Data.Array as Array
import Data.Either (Either(..))
import Data.Maybe (Maybe(..), fromMaybe)
import Data.String as String
import Data.String.Pattern (Pattern(..))
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Host.Client as Report
import FluentCI.Host.Plan as Plan
import FluentCI.Plugin as P
import Foreign.Object as Object

type Runtime = { project :: String, runDir :: String, docker :: String, timeoutMs :: Int, experimental :: Boolean }

decodeObject :: Array String -> Json -> Either String (Object.Object Json)
decodeObject allowed json = case decodeJson json of
  Left error -> Left (show error)
  Right object -> case Array.find (\key -> not (Array.elem key allowed)) (Object.keys object) of
    Just key -> Left ("Unknown Buildx module argument: " <> key)
    Nothing -> Right object

optional :: forall a. DecodeJson a => Object.Object Json -> String -> Either String (Maybe a)
optional object key = case getFieldOptional object key of
  Left error -> Left ("Invalid argument " <> key <> ": " <> show error)
  Right value -> Right value

field :: forall a. DecodeJson a => Object.Object Json -> String -> a -> Either String a
field object key fallback = fromMaybe fallback <$> optional object key

stringOption :: String -> Maybe String -> Array String
stringOption flag = case _ of
  Nothing -> []
  Just value -> [ "--" <> flag <> "=" <> value ]

booleanOption :: String -> Maybe Boolean -> Array String
booleanOption flag = case _ of
  Nothing -> []
  Just value -> [ "--" <> flag <> "=" <> if value then "true" else "false" ]

intOption :: String -> Maybe Int -> Array String
intOption flag = case _ of
  Nothing -> []
  Just value -> [ "--" <> flag <> "=" <> show value ]

arrayOption :: String -> Maybe (Array String) -> Array String
arrayOption flag = case _ of
  Nothing -> []
  Just values -> map (\value -> "--" <> flag <> "=" <> value) values

positional :: Array String -> Array String
positional values = if Array.null values then [] else [ "--" ] <> values

maybePositional :: Maybe String -> Array String
maybePositional = case _ of
  Nothing -> []
  Just value -> [ value ]

validateRuntime :: Runtime -> Array String -> Either String Unit
validateRuntime config arguments
  | config.project == "" || config.runDir == "" || config.docker == "" = Left "project, runDir and docker must be nonempty"
  | config.timeoutMs <= 0 = Left "timeoutMs must be a positive integer"
  | Array.any (String.contains (Pattern "\x0000")) ([ config.project, config.runDir, config.docker ] <> arguments) = Left "Arguments cannot contain NUL"
  | otherwise = Right unit

commandPlan :: Runtime -> Array String -> Either String Plan.Plan
commandPlan config arguments = do
  validateRuntime config arguments
  pure $ Plan.plan
    [ Plan.task "buildx" config.docker config.project
        # Plan.withArgs arguments
        # Plan.withTimeout config.timeoutMs
        # Plan.withEnv "BUILDX_EXPERIMENTAL" (if config.experimental then "1" else "0")
    ] # Plan.withInheritedEnv true # Plan.withJobs 1

require :: forall a. P.Result a -> Effect a
require = case _ of
  Left error -> throw (P.errorMessage error)
  Right value -> pure value

execute :: Runtime -> Plan.Plan -> Effect String
execute config plan = do
  report <- P.runPlan config.runDir plan >>= require
  unless (report.outcome == Just Report.RunSucceeded)
    (throw ("Docker Buildx failed; inspect " <> config.runDir <> "/result.json"))
  P.pipeline "buildx-result"
    # P.exec [ "/bin/cat", config.runDir <> "/result.json" ]
    # P.stdout
    # (_ >>= require)
