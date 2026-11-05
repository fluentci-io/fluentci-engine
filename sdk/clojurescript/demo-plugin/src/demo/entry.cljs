(ns demo.entry (:require [demo.plugin :as plugin]))

;; Explicit CommonJS boundary keeps guest exports stable through compilation.
(set! (.-exports js/module)
      #js {:greet plugin/greet :echo plugin/echo :failure plugin/failure
           :hash_file plugin/hash-file :copy_file plugin/copy-file :nested plugin/nested})
