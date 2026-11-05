(ns ^:typed.clojure fluentci.plugin
  "Immutable, thread-first guest pipelines. Only functions ending in ! do I/O."
  (:refer-clojure :exclude [file run!])
  (:require [clojure.string :as str]
            [fluentci.bridge :as bridge]
            #?(:clj [typed.clojure :as t])))

#?(:clj
   (do
     (t/defalias Step
       (t/U (t/HMap :mandatory {:op ':exec :argv (t/Vec t/Str)})
            (t/HMap :mandatory {:op ':shell :command t/Str})
            (t/HMap :mandatory {:op ':workdir :path t/Str})
            (t/HMap :mandatory {:op ':env :key t/Str :value t/Str})
            (t/HMap :mandatory {:op ':cache :key t/Str :path t/Str})
            (t/HMap :mandatory {:op ':secret :key t/Str :name t/Str :value t/Str})))
     (t/defalias Pipeline
       (t/HMap :mandatory {:kind ':pipeline :name t/Str :steps (t/Vec Step)}))
     (t/defalias File (t/HMap :mandatory {:kind ':file :path t/Str}))
     (t/defalias Secret (t/HMap :mandatory {:kind ':secret :name t/Str :value t/Str}))))

#?(:clj (t/ann pipeline [t/Str -> Pipeline]))
(defn pipeline [name] {:kind :pipeline :name name :steps []})

#?(:clj (t/ann exec [Pipeline (t/Vec t/Str) -> Pipeline]))
(defn exec
  "Append a literal argv vector. Shell syntax is never interpreted."
  [pipeline argv]
  (when (or (empty? argv) (empty? (first argv)) (some #(re-find #"\x00" %) argv))
    (throw (ex-info "exec requires a program and NUL-free arguments" {:kind :invalid-argument})))
  (update pipeline :steps conj {:op :exec :argv argv}))

#?(:clj (t/ann shell [Pipeline t/Str -> Pipeline]))
(defn shell
  "Explicitly append shell syntax. Never interpolate untrusted input."
  [pipeline command]
  (update pipeline :steps conj {:op :shell :command command}))

#?(:clj (t/ann workdir [Pipeline t/Str -> Pipeline]))
(defn workdir [pipeline path]
  (update pipeline :steps conj {:op :workdir :path path}))

#?(:clj (t/ann env [Pipeline t/Str t/Str -> Pipeline]))
(defn env [pipeline key value]
  (update pipeline :steps conj {:op :env :key key :value value}))

#?(:clj (t/ann cache [Pipeline t/Str t/Str -> Pipeline]))
(defn cache [pipeline key path]
  (update pipeline :steps conj {:op :cache :key key :path path}))

#?(:clj (t/ann file [t/Str -> File]))
(defn file [path] {:kind :file :path path})

#?(:clj (t/ann secret [t/Str t/Str -> Secret]))
(defn secret [name value] {:kind :secret :name name :value value})

#?(:clj (t/ann with-secret [Pipeline t/Str Secret -> Pipeline]))
(defn with-secret [pipeline key secret]
  (update pipeline :steps conj {:op :secret :key key :name (:name secret) :value (:value secret)}))

#?(:clj (t/ann copy-file [Pipeline File t/Str -> Pipeline]))
(defn copy-file [pipeline source destination]
  (exec pipeline ["/bin/cp" "--" (:path source) destination]))

#?(:clj (t/ann quote-arg [t/Str -> t/Str]))
(defn- quote-arg [value]
  (str "'" (str/replace value "'" "'\\''") "'"))

#?(:clj (t/ann emit-step! [Step -> nil]))
(defn- emit-step! [step]
  (case (:op step)
    :exec (bridge/call! "with_exec" (bridge/json (mapv quote-arg (:argv step))) false)
    :shell (bridge/call! "with_exec" (bridge/json [(:command step)]) false)
    :workdir (bridge/call! "with_workdir" (:path step) false)
    :env (bridge/call! "set_envs" (bridge/json [[(:key step) (:value step)]]) false)
    :cache (let [id (bridge/handle (bridge/call! "cache" (:key step) true))]
             (bridge/call! "with_cache" (bridge/json {:id id :key (:key step) :path (:path step)}) false))
    :secret (let [id (bridge/call! "set_secret" (bridge/json [(:name step) (:value step)]) true)]
              (bridge/call! "with_secret_variable" (bridge/json [(:key step) id]) false)))
  nil)

#?(:clj (t/ann run! [Pipeline t/Bool -> t/Str]))
(defn- run! [pipeline stderr?]
  (bridge/call! "set_runner" "default" false)
  (bridge/call! "pipeline" (:name pipeline) true)
  (doseq [step (:steps pipeline)] (emit-step! step))
  (bridge/call! (if stderr? "stderr" "stdout") nil true))

#?(:clj (t/ann stdout! [Pipeline -> t/Str]))
(defn stdout! [pipeline] (run! pipeline false))

#?(:clj (t/ann stderr! [Pipeline -> t/Str]))
(defn stderr! [pipeline] (run! pipeline true))

#?(:clj (t/ann sha256! [File -> t/Str]))
(defn sha256! [file]
  (bridge/call! "file" (:path file) true)
  (bridge/call! "sha256" (:path file) true))

#?(:clj (t/ann md5! [File -> t/Str]))
(defn md5! [file]
  (bridge/call! "file" (:path file) true)
  (bridge/call! "md5" (:path file) true))

#?(:clj (t/ann call-module! [t/Str t/Str t/Str -> t/Str]))
(defn call-module! [url function input]
  (bridge/call! "call" (bridge/json {:url url :function function :args input}) true))

#?(:clj (t/ann input! [-> t/Str]))
(defn input! [] (bridge/input!))

#?(:clj (t/ann output! [t/Str -> nil]))
(defn output! [value] (bridge/output! value))
