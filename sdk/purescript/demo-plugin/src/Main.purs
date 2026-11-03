module Main where

import Prelude
import Data.Either (Either(..))
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Plugin as P
import FluentCI.Host.Plan as Plan

unwrap :: forall a. P.Result a -> Effect a
unwrap result = case result of
  Left error -> throw (P.errorMessage error)
  Right value -> pure value

send :: String -> Effect Unit
send text = P.output text >>= unwrap

request :: Effect String
request = P.input >>= unwrap

run :: P.Pipeline -> Effect Unit
run value = P.stdout value >>= unwrap >>= send

greet :: Effect Unit
greet = do
  name <- request
  P.pipeline "purescript-greeting"
    # P.exec ["/usr/bin/printf", "%s", name]
    # run

echo :: Effect Unit
echo = request >>= send

failure :: Effect Unit
failure = P.pipeline "expected-failure" # P.exec ["/bin/sh", "-c", "exit 7"] # run

invalid :: Effect Unit
invalid = P.pipeline "invalid" # P.exec [] # run

hashFile :: Effect Unit
hashFile = request >>= (P.sha256 <<< P.file) >>= unwrap >>= send

copyFile :: Effect Unit
copyFile = do
  source <- request
  P.pipeline "copy-file"
    # P.copyFile "copied file.txt" (P.file source)
    # P.exec ["/bin/cat", "copied file.txt"]
    # run

nested :: Effect Unit
nested = do
  name <- request
  P.callModule "plugin.wasm" "echo" name >>= unwrap >>= send

secretEnv :: Effect Unit
secretEnv = do
  value <- request
  P.pipeline "secret-env"
    # P.withSecret "TOKEN" (P.secret "TOKEN" value)
    # P.exec ["/usr/bin/python3", "-c", "import os; assert os.environ['TOKEN']; print('present')"]
    # run

serviceValue :: P.Service
serviceValue = P.pipeline "background"
  # P.exec ["/bin/sh", "-c", "echo $$ > service.pid; touch service.ready; exec sleep 120"]
  # P.asService "background"

service :: Effect Unit
service = P.pipeline "client"
  # P.withService serviceValue
  # P.exec ["/bin/sh", "-c", "test -f service.ready && printf healthy"]
  # run

cancel :: Effect Unit
cancel = P.pipeline "cancel-client"
  # P.withService serviceValue
  # P.exec ["/bin/sh", "-c", "echo $$ > client.pid; touch client.ready; exec sleep 120"]
  # run

-- Pure host-plan construction is also safe inside a guest; the caller chooses
-- when and where to execute the returned plan through the durable run API.
actionPlan :: Effect Unit
actionPlan = do
  let
    task = Plan.task "build" "/bin/sh" "."
      # Plan.withArgs ["-c", "/bin/cat seed.txt > output.txt; printf run >> executions"]
      # Plan.withCache { inputs: ["seed.txt"], outputs: ["output.txt"], toolchain: "purescript-guest-v1" }
    plan = Plan.plan [task]
  case Plan.compile plan of
    Left error -> throw (show error)
    Right encoded -> send encoded
