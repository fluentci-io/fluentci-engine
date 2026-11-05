(ns check (:require [typed.clojure :as t]))

(defn -main [& namespaces]
  (doseq [namespace (concat '[fluentci.bridge fluentci.plugin]
                           (if (seq namespaces) (map symbol namespaces) '[demo.plugin]))]
    (t/check-ns-clj namespace))
  (doseq [namespace '[fluentci.type-errors.argv fluentci.type-errors.resource
                     fluentci.type-errors.transform fluentci.type-errors.chain]]
    (let [diagnostics (java.io.StringWriter.)
          failure (binding [*out* diagnostics *err* diagnostics]
                    (try (t/check-ns-clj namespace) nil
                         (catch clojure.lang.ExceptionInfo e e)))]
      (when-not (and failure (re-find #"Type Checker: Found" (.getMessage failure)))
        (print (str diagnostics))
        (throw (ex-info (str "Expected static type errors in " namespace) {} failure)))))
  (println "SDK and example passed static type checking"))
