module FluentCI.Secret where

import Prelude

import Effect.Aff (Aff)
import Effect.Aff.Compat (EffectFnAff, fromEffectFnAff)


data Secret


foreign import _plaintext :: Secret -> EffectFnAff String

plaintext :: Secret -> Aff String
plaintext secret = fromEffectFnAff $ _plaintext secret