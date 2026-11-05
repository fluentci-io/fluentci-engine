(ns fluentci.test-runner
  (:require [cljs.test :as test :refer-macros [deftest is run-tests]]
            [fluentci.plugin :as p]))

(defn guest []
  (let [values (js/Map.)
        counter (atom 0)
        calls (atom [])
        allocate (fn [value]
                   (let [offset (js/BigInt (swap! counter inc))]
                     (.set values offset value)
                     offset))
        functions #js {}]
    (set! (.-Memory js/globalThis)
          #js {:fromString (fn [value] #js {:offset (allocate value)})
               :find (fn [offset] #js {:readString (fn [] (.get values offset))})})
    (doseq [name ["set_runner" "pipeline" "with_exec" "with_workdir" "set_envs"
                 "cache" "with_cache" "set_secret" "with_secret_variable"
                 "file" "sha256" "stdout" "stderr"]]
      (aset functions name
            (fn [& offsets]
              (when-let [offset (first offsets)]
                (is (= "bigint" (js* "typeof ~{}" offset))))
              (swap! calls conj {:function name :input (.get values (first offsets))})
              (case name
                ("pipeline" "cache") (allocate "{\"id\":\"handle\"}")
                "stdout" (allocate "héllo 世界")
                "set_secret" (allocate "secret-id")
                "sha256" (allocate "digest")
                "stderr" (throw (js/Error. "engine failure"))
                nil))))
    (set! (.-Host js/globalThis) #js {:getFunctions (fn [] functions)})
    calls))

(defn payload [calls function]
  (->> @calls (filter #(= function (:function %))) first :input))

(deftest pure-composable-reusable-pipelines
  (let [calls (guest)
        base (p/pipeline "build")
        release #(p/env % "MODE" "release")
        value "spaces 'quotes'; $(touch injected) 世界"
        configured (-> base release (p/exec ["printf" "%s" value]))]
    (is (= [] (:steps base)))
    (is (= [] @calls))
    (is (= "héllo 世界" (p/stdout! configured)))
    (is (= ["'printf'" "'%s'" "'spaces '\\''quotes'\\''; $(touch injected) 世界'"]
           (js->clj (.parse js/JSON (payload calls "with_exec")))))
    (is (= [["MODE" "release"]] (js->clj (.parse js/JSON (payload calls "set_envs")))))
    (let [first-run @calls]
      (reset! calls [])
      (p/stdout! configured)
      (is (= first-run @calls)))))

(deftest explicit-shell-and-canonical-payloads
  (let [calls (guest)]
    (-> (p/pipeline "build")
        (p/workdir "some directory")
        (p/cache "cargo" "target")
        (p/with-secret "TOKEN" (p/secret "token" "sensitive"))
        (p/shell "printf hello && printf world")
        p/stdout!)
    (is (= "some directory" (payload calls "with_workdir")))
    (is (= {"id" "handle" "key" "cargo" "path" "target"}
           (js->clj (.parse js/JSON (payload calls "with_cache")))))
    (is (= ["TOKEN" "secret-id"] (js->clj (.parse js/JSON (payload calls "with_secret_variable")))))
    (is (= ["printf hello && printf world"] (js->clj (.parse js/JSON (payload calls "with_exec")))))))

(deftest validation-precedes-effects
  (let [calls (guest)]
    (doseq [argv [[] [""] ["printf" "bad\u0000argument"]]]
      (is (thrown? cljs.core/ExceptionInfo (p/exec (p/pipeline "bad") argv))))
    (is (= [] @calls))))

(deftest errors-carry-host-function-context
  (guest)
  (try
    (p/stderr! (p/pipeline "failure"))
    (is false "expected a host error")
    (catch cljs.core/ExceptionInfo e
      (is (= {:kind :host-error :function "stderr"} (ex-data e)))
      (is (re-find #"engine failure" (ex-message e))))))

(deftest file-hashes-use-canonical-abi
  (let [calls (guest)]
    (is (= "digest" (p/sha256! (p/file "a file.txt"))))
    (is (= "a file.txt" (payload calls "sha256"))))
  (let [calls (guest)]
    (set! (.-Host js/globalThis) #js {:getFunctions (fn [] #js {})})
    (is (thrown? cljs.core/ExceptionInfo (p/stdout! (p/pipeline "missing"))))
    (is (empty? @calls))))

(defmethod test/report [::test/default :end-run-tests] [summary]
  (set! (.-exitCode js/process) (if (test/successful? summary) 0 1)))

(defn -main [] (run-tests 'fluentci.test-runner))
(set! *main-cli-fn* -main)
