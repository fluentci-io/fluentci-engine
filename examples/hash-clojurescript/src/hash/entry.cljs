(ns hash.entry (:require [hash.plugin :as plugin]))
(set! (.-exports js/module) #js {:md5 plugin/md5 :sha256 plugin/sha256})
