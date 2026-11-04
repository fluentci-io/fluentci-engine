module CI.E2E (check, buildWasm, providers, typescript, graphql) where

import Prelude
import CI.Rust as Rust
import CI.Support (run, runIn, unwrap)
import Data.Foldable (foldl)
import Effect (Effect)
import FluentCI.Plugin as P

check :: Effect Unit
check = do
  Rust.build
  buildWasm
  providers
  typescript
  graphql

buildWasm :: Effect Unit
buildWasm = run "provider-plugins"
  [ [ "rustup", "target", "add", "wasm32-unknown-unknown" ]
  , [ "cargo", "build", "--release", "--locked", "--target", "wasm32-unknown-unknown", "-p", "archive", "-p", "chmod", "-p", "devbox", "-p", "flox", "-p", "git", "-p", "hash", "-p", "hermit", "-p", "http", "-p", "mise", "-p", "nix", "-p", "pixi", "-p", "pkgx", "-p", "proto" ]
  ]

providers :: Effect Unit
providers = runIn "provider-smoke-tests" "fixtures"
  [ [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/flox.wasm", "exec which jq" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/archive.wasm", "tar_czvf flox-demo/.flox" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/hash.wasm", "md5 flox-demo/.flox.tar.gz" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/hash.wasm", "sha256 flox-demo/.flox.tar.gz" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/nix.wasm", "exec which deno" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/devbox.wasm", "exec which gh" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/git.wasm", "git_tree https://github.com/tsirysndr/me" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/mise.wasm", "exec which bun" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/pixi.wasm", "exec which php" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/pkgx.wasm", "exec which deno" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/hermit.wasm", "exec which jq" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/chmod.wasm", "chmod a+x hello.sh" ]
  , [ "../target/release/fluentci-engine", "call", "-m", "../target/wasm32-unknown-unknown/release/http.wasm", "get https://fluentci.io" ]
  ]

typescript :: Effect Unit
typescript = run "typescript-sdk"
  [ [ "target/release/fluentci-engine", "run", "--", "deno", "run", "-A", "sdk/typescript/demo/main.ts" ] ]

graphql :: Effect Unit
graphql = do
  let service = P.pipeline "engine-api"
        # P.withEnv "FLUENTCI_ENGINE_HOST" "127.0.0.1"
        # P.exec [ "target/release/fluentci-engine", "serve" ]
        # P.asService "fluentci-engine"
  let client = P.pipeline "graphql-provider-tests"
        # P.using P.Pkgx
        # P.withService service
        # P.withPackages [ "httpie" ]
        # P.withWorkdir "fixtures"
        # P.waitOn 6880 60
  let queries = [ "zip", "tar-czvf", "unzip", "tar-xzvf", "md5", "sha256", "git", "directory", "envhub", "mise", "pixi", "pkgx", "nix", "flox", "devenv", "devbox", "proto" ]
  let request pipeline query = pipeline # P.shell
        ("""http POST http://127.0.0.1:6880/graphql Content-Type:application/json query="$(cat """ <> query <> """.graphql)" --ignore-stdin --pretty format""")
  P.stdout (foldl request client queries) >>= unwrap >>= P.output >>= unwrap
