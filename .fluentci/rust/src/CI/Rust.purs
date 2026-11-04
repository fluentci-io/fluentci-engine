module CI.Rust (test, build, clippy, format, llvmCov) where

import Prelude
import CI.Support (run)
import Effect (Effect)

test :: Effect Unit
test = run "rust-tests"
  [ [ "cargo", "test", "--release", "--locked", "-p", "fluentci-engine", "-p", "fluentci-executor", "-p", "fluentci-core", "-p", "fluentci-server", "--lib", "--tests" ] ]

build :: Effect Unit
build = run "rust-build" [ [ "cargo", "build", "--release", "--locked", "-p", "fluentci-engine" ] ]

clippy :: Effect Unit
clippy = run "rust-clippy" [ [ "cargo", "clippy", "--release", "--locked", "-p", "fluentci-executor", "--all-targets", "--", "-D", "warnings" ] ]

format :: Effect Unit
format = run "rust-format" [ [ "cargo", "fmt", "--all", "--", "--check" ] ]

llvmCov :: Effect Unit
llvmCov = run "rust-coverage" [ [ "cargo", "llvm-cov", "--release", "--locked", "--workspace", "--lib", "--lcov", "--output-path", "lcov.info" ] ]
