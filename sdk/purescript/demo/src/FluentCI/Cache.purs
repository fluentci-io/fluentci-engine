module FluentCI.Cache where

import Prelude
import Effect.Aff (Aff)
import Effect.Aff.Compat (EffectFnAff, fromEffectFnAff)

data Cache

foreign import _id :: Cache -> EffectFnAff String

foreign import _key :: Cache -> EffectFnAff String

id :: Cache -> Aff String
id cache = fromEffectFnAff $ _id cache

key :: Cache -> Aff String
key cache = fromEffectFnAff $ _key cache