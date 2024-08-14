# FluentCI Purescript SDK

The FluentCI Purescript SDK is a library that provides a Purescript interface to the FluentCI API.

Go to the [purescript-fluentci](https://github.com/fluentci-io/purescript-fluentci) repository for more information.

## 🚚 Installation

Add `fluentci` to your `spago.dhall` dependencies:

```dhall
{ name = "demo"
, dependencies = [ "aff", "console", "effect", "prelude", "fluentci" ]
, packages = ./packages.dhall
, sources = [ "src/**/*.purs", "test/**/*.purs" ]
}
```

And add the following to your `packages.dhall`:

```dhall
let upstream =
      https://github.com/purescript/package-sets/releases/download/psc-0.15.15-20240812/packages.dhall
        sha256:557dc416167543923d0f944fdc0c7bc3e0466b9467a8fc2fba0460f8d5425725

in  upstream
  with fluentci =
    { dependencies =
       [ "aff"
       , "console"
       , "effect"
       , "prelude"
       ]
    , repo =
        "https://github.com/fluentci-io/purescript-fluentci.git"
    , version =
        "d0d904915b1eade1f1e72ab541708ad4b43ccd5e"
    }
```

Then install the package:

```bash
spago install
```

## 🚀 Quick Start

This is a quick start guide to get you up and running with the FluentCI Purescript SDK.

```purescript
module Main where

import Prelude

import Effect (Effect)
import Effect.Aff (launchAff_)
import Effect.Class (liftEffect)
import Effect.Class.Console as Console
import FluentCI.Class (asService, id, mise, nix, pkgx, stdout, withExec, withSecretVariable, withService, withWorkdir)
import FluentCI.Client (cache, dag, git, pipeline, setSecret)
import FluentCI.Directory (Directory, entries)
import FluentCI.Git (branch, tree)
import FluentCI.Mise (Mise)
import FluentCI.Pipeline (Pipeline)
import FluentCI.Pkgx (withPackages)
import FluentCI.Secret (Secret, plaintext)
import FluentCI.Service (Service)

main :: Effect Unit
main = launchAff_ do
  secret <- liftEffect $ setSecret dag "GITHUB" "my-github-token"
  plaintext secret >>= Console.log
  p <- liftEffect $ secretDemo secret
  stdout p >>= Console.log

  m <- liftEffect $ miseDemo
  stdout m >>= Console.log

  c <- liftEffect $ cache dag "pixi"
  id c >>= Console.log

  pingService <- liftEffect $ ping
  pingGhService <- liftEffect $ pingGh

  pingDemoPipeline <- liftEffect $ pingDemo pingService pingGhService
  stdout pingDemoPipeline >>= Console.log

  git <- liftEffect $ gitDemo
  stdout git >>= Console.log

  gitEntries <- liftEffect $ gitEntriesDemo
  entries gitEntries >>= Console.debugShow

ping :: Effect Service
ping = do
  p <- pipeline dag "demo"
  p1 <- nix p { impure: true }
  p2 <- withWorkdir p1 "./nix-demo"
  p3 <- withExec p2 ["nix", "--version"]
  p4 <- withExec p3 ["ping", "fluentci.io"]
  asService p4 "ping-fluentci"

secretDemo :: Secret -> Effect Pipeline
secretDemo secret = do
  p <- pipeline dag "secret-demo"
  p1 <- withSecretVariable p "GITHUB" secret
  withExec p1 ["echo", "$GITHUB"]

pingGh :: Effect Service
pingGh = do
  p <- pipeline dag "demo"
  p1 <- pkgx p
  p2 <- withPackages p1 ["ping"]
  p3 <- withExec p2 ["ping", "github.com"]
  asService p3 "ping-github"

miseDemo :: Effect Mise
miseDemo = do
  p <- pipeline dag "mise-demo"
  m <- mise p
  m1 <- withWorkdir m "./mise-demo"
  m2 <- withExec m1 ["mise", "--version"]
  withExec m2 ["which", "bun"]

pingDemo :: Service -> Service -> Effect Pipeline
pingDemo svc1 svc2 = do
  p <- pipeline dag "ping-demo"
  p1 <- withService p svc1
  p2 <- withService p1 svc2
  p3 <- withExec p2 ["sleep", "15"]
  withExec p3 ["ls", "-ltr", ".fluentci"]

gitDemo :: Effect Directory
gitDemo = do
  g <- git dag "https://github.com/tsirysndr/me"
  g1 <- branch g "main"
  g2 <- tree g1
  g3 <- withExec g2 ["ls", "-ltr"]
  withExec g3 ["pwd"]

gitEntriesDemo :: Effect Directory
gitEntriesDemo = do
  g <- git dag "https://github.com/tsirysndr/me"
  g1 <- branch g "main"
  tree g1
```
