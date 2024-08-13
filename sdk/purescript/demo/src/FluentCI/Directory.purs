module FluentCI.Directory where

import Prelude

import Effect.Aff (Aff)
import Effect.Aff.Compat (EffectFnAff, fromEffectFnAff)
import FluentCI.Cache (Cache)
import FluentCI.Devbox (Devbox)
import FluentCI.Devenv (Devenv)
import FluentCI.Flox (Flox)
import FluentCI.Mise (Mise)
import FluentCI.Nix (Nix)
import FluentCI.Pixi (Pixi)
import FluentCI.Pkgx (Pkgx)
import FluentCI.Secret (Secret)
import FluentCI.Service (Service)


data Directory

data File

foreign import _id :: Directory -> EffectFnAff String

foreign import _stderr :: Directory -> EffectFnAff String

foreign import _stdout :: Directory -> EffectFnAff String

foreign import _entries :: Directory -> EffectFnAff (Array String)

id :: Directory -> Aff String
id dir = fromEffectFnAff $ _id dir

stderr :: Directory -> Aff String
stderr dir = fromEffectFnAff $ _stderr dir

stdout :: Directory -> Aff String
stdout dir = fromEffectFnAff $ _stdout dir

entries :: Directory -> Aff (Array String)
entries dir = fromEffectFnAff $ _entries dir

foreign import devbox :: Directory -> Aff Devbox

foreign import devenv :: Directory -> Aff Devenv

foreign import flox :: Directory -> Aff Flox

foreign import mise :: Directory -> Aff Mise

foreign import nix :: Directory -> Aff Nix

foreign import pixi :: Directory -> Aff Pixi

foreign import pkgx :: Directory -> Aff Pkgx

foreign import directory :: Directory -> String -> Aff Directory

foreign import tarCzvf :: Directory  -> Aff File

foreign import waitOn :: Directory -> Int -> Int -> Aff Directory

foreign import withCache :: Directory -> Cache -> Aff Directory

foreign import withEnvVariable :: Directory -> String -> String -> Aff Directory

foreign import withExec :: Directory -> Array String -> Aff Directory

foreign import withFile :: Directory -> String -> Aff Directory

foreign import withSecretVariable :: Directory -> String -> Secret -> Aff Directory

foreign import withService :: Directory -> Service -> Aff Directory

foreign import withWorkdir :: Directory -> String -> Aff Directory

foreign import zip :: Directory -> Aff File