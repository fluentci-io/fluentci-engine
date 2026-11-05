(ns ^:typed.clojure demo.plugin
  (:require [fluentci.plugin :as p]
            #?(:clj [typed.clojure :as t])))

;; Reusable pipeline transformations are ordinary typed functions.
#?(:clj (t/ann release [p/Pipeline -> p/Pipeline]))
(defn release [pipeline] (p/env pipeline "MODE" "release"))

#?(:clj (t/ann greeting [t/Str -> p/Pipeline]))
(defn greeting [name]
  (-> (p/pipeline "clojurescript-greeting")
      release
      (p/exec ["/usr/bin/printf" "%s" name])))

#?(:clj (t/ann greet [-> nil]))
(defn greet [] (-> (p/input!) greeting p/stdout! p/output!))

#?(:clj (t/ann echo [-> nil]))
(defn echo [] (-> (p/input!) p/output!))

#?(:clj (t/ann failure [-> nil]))
(defn failure []
  (-> (p/pipeline "expected-failure")
      (p/exec ["/bin/sh" "-c" "exit 7"])
      p/stdout! p/output!))

#?(:clj (t/ann hash-file [-> nil]))
(defn hash-file [] (-> (p/input!) p/file p/sha256! p/output!))

#?(:clj (t/ann copy-file [-> nil]))
(defn copy-file []
  (-> (p/pipeline "copy-file")
      (p/copy-file (p/file (p/input!)) "copied file.txt")
      (p/exec ["/bin/cat" "copied file.txt"])
      p/stdout! p/output!))

#?(:clj (t/ann nested [-> nil]))
(defn nested [] (p/output! (p/call-module! "plugin.wasm" "echo" (p/input!))))
