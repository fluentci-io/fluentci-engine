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
  value <- plaintext secret
  p <- liftEffect $ secretDemo secret
  output <- stdout p

  m <- liftEffect $ miseDemo
  miseOutput <- stdout m

  c <- liftEffect $ cache dag "pixi"
  cacheId <- id c

  pingService <- liftEffect $ ping
  pingGhService <- liftEffect $ pingGh

  pingDemoPipeline <- liftEffect $ pingDemo pingService pingGhService
  pingOutput <- stdout pingDemoPipeline

  git <- liftEffect $ gitDemo
  gitOutput <- stdout git

  gitEntries <- liftEffect $ gitEntriesDemo
  gitEntriesOutput <- entries gitEntries

  Console.log value
  Console.log "Secret demo:"
  Console.log output

  Console.log "Mise: "
  Console.log miseOutput

  Console.log "CacheId: "
  Console.log cacheId

  Console.log "pingOutput: "
  Console.log pingOutput

  Console.log "GitOutput: "
  Console.log gitOutput

  Console.log "GitEntriesOutput: "
  Console.debugShow gitEntriesOutput

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
