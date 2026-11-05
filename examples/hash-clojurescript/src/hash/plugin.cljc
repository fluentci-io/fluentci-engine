(ns ^:typed.clojure hash.plugin
  "A complete port of examples/hash, using the ClojureScript SDK."
  (:require [fluentci.plugin :as p]
            #?(:clj [typed.clojure :as t])))

#?(:clj (t/ann md5 [-> nil]))
(defn md5 [] (-> (p/input!) p/file p/md5! p/output!))

#?(:clj (t/ann sha256 [-> nil]))
(defn sha256 [] (-> (p/input!) p/file p/sha256! p/output!))
