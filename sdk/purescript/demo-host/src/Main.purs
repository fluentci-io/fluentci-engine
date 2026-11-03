module Main where

import Prelude
import Data.Either (Either(..))
import Effect (Effect)
import Effect.Console (log)
import Effect.Exception (throw)
import FluentCI.Host.Plan as Host

main :: Effect Unit
main = do
  let
    build = Host.task "build" "/bin/sh" "."
      # Host.withArgs [ "-c", "/bin/cat input > output; printf built" ]
      # Host.withCache { inputs: [ "input" ], outputs: [ "output" ], toolchain: "host-demo-v1" }
      # Host.withToolchainFiles [ "/bin/cat" ]
    pipeline = Host.plan [build] # Host.withJobs 2 # Host.withCacheDirectory "cache"
  case Host.compile pipeline of
    Left error -> throw (show error)
    Right json -> log json
