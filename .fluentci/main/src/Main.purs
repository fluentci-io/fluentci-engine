module Main (check, test, build, e2e) where

import Prelude
import CI.E2E as E2E
import CI.Rust as Rust
import Effect (Effect)

check :: Effect Unit
check = Rust.test *> E2E.check

test :: Effect Unit
test = Rust.test

build :: Effect Unit
build = Rust.build

e2e :: Effect Unit
e2e = E2E.check
