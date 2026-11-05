(ns ^:typed.clojure demo.plugin
  (:require [fluentci.plugin :as p]
            [fluentci.host :as h]
            [fluentci.files :as f]
            [fluentci.matrix :as m]
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

#?(:clj (t/ann finish! [h/Plan t/Str t/Str -> nil]))
(defn- finish! [plan directory message]
  (if (h/succeeded? (p/run-plan! plan directory))
    (p/output! message)
    (throw (ex-info "Plan failed; inspect result.json" {:kind :plan-failed}))))

#?(:clj (t/ann matrix [-> nil]))
(defn matrix []
  (let [directory (p/input!)
        prepare (-> (h/task "prepare" "/usr/bin/python3" ".")
                    (h/with-args ["-c" "from pathlib import Path; Path('seed.txt').write_text('seed')"])
                    (h/with-cache (h/cache ["mode.txt"] ["seed.txt"] "matrix-prepare-v1")))
        cases (-> (m/product (m/axis "node" ["22" "24"]) (m/axis "suite" ["unit" "integration"]))
                  (m/exclude (fn [pair] (= pair ["22" "integration"])))
                  (m/include [(m/named "experimental" ["26" "unit"])]))
        jobs (m/matrix "test" cases
               (fn [[node suite]]
                 (-> (h/task "assigned-by-matrix" "/usr/bin/python3" ".")
                     (h/with-args ["job.py" node suite])
                     (h/with-cache (h/cache ["seed.txt" "mode.txt" "job.py"] [(str "out-" node "-" suite)] "matrix-v1"))
                     (h/after [prepare]))))
        join (-> (h/task "join" "/usr/bin/python3" ".")
                 (h/with-args ["-c" "from pathlib import Path; Path('joined').write_text('done')"])
                 (h/after [prepare]) (h/after jobs))]
    (finish! (-> (m/parallel (into [prepare] (conj jobs join)) 2)
                 (h/with-fail-fast false) (h/with-cache-directory "action-cache"))
             directory "matrix complete")))

#?(:clj (t/ann workflow-job [t/Str t/Str -> h/Task]))
(defn- workflow-job [id mode]
  (-> (h/task id "/usr/bin/python3" ".") (h/with-args ["job.py" mode])))

#?(:clj (t/ann workflow [-> nil]))
(defn workflow []
  (let [directory (p/input!)
        producer (-> (workflow-job "produce" "produce") (h/retry 3 10)
                     (h/output "version" "value.txt") (h/artifact "binary" "artifact.bin") (h/resource "checkout"))
        consumer (-> (workflow-job "consume" "consume") (h/output-env "VALUE" producer "version"))
        optional (-> (workflow-job "optional" "optional") (h/allow-failure true))
        never (-> (workflow-job "never" "never") (h/when-task :never))
        failure (-> (workflow-job "failure" "failure") (h/after [consumer]) (h/when-task :failure))
        cleanup (-> (workflow-job "cleanup" "cleanup") (h/after [consumer optional never failure]) h/always)]
    (finish! (-> (h/plan [producer consumer optional never failure cleanup]) (h/with-jobs 2)
                 (h/with-resource-directory "leases"))
             directory "workflow complete")))

#?(:clj (t/ann files [-> nil]))
(defn files []
  (let [directory (p/input!)
        dir (f/mkdir "directory" "data" true ".")
        write (-> (f/write-text "write" "data/version.env" "VERSION='héllo 世界; $(touch injected)'\n" false ".") (h/after [dir]))
        copy (-> (f/copy-file "copy" "data/version.env" "copied.env" false ".") (h/after [write]))
        read (-> (f/read-text "read" "copied.env" 1048576 ".") (h/after [copy]))
        hash (-> (f/hash "hash" "copied.env" ".") (h/after [copy]))
        env (-> (h/task "env" "/usr/bin/printenv" ".") (h/with-args ["VERSION"])
                (h/env-file "copied.env") (h/after [copy]))]
    (finish! (-> (h/plan [dir write copy read hash env]) (h/with-jobs 2)) directory "files complete")))

#?(:clj (t/ann background [-> p/Service]))
(defn- background []
  (-> (p/pipeline "background")
      (p/exec ["/bin/sh" "-c" "echo $$ > service.pid; touch service.ready; exec sleep 120"])
      (p/as-service "background")))
#?(:clj (t/ann service [-> nil]))
(defn service []
  (-> (p/pipeline "client") (p/with-service (background))
      (p/exec ["/bin/sh" "-c" "test -f service.ready && printf healthy"])
      p/stdout! p/output!))
#?(:clj (t/ann cancel [-> nil]))
(defn cancel []
  (-> (p/pipeline "cancel-client") (p/with-service (background))
      (p/exec ["/bin/sh" "-c" "echo $$ > client.pid; touch client.ready; exec sleep 120"])
      p/stdout! p/output!))
#?(:clj (t/ann secret-env [-> nil]))
(defn secret-env []
  (-> (p/pipeline "secret-env") (p/with-secret "TOKEN" (p/secret "TOKEN" (p/input!)))
      (p/exec ["/usr/bin/python3" "-c" "import os; assert os.environ['TOKEN']; print('present')"])
      p/stdout! p/output!))
#?(:clj (t/ann action-plan [-> nil]))
(defn action-plan []
  (-> (h/plan [(-> (h/task "build" "/bin/sh" ".")
                  (h/with-args ["-c" "/bin/cat seed.txt > output.txt; printf run >> executions"])
                  (h/with-cache (h/cache ["seed.txt"] ["output.txt"] "clojurescript-guest-v1")))])
      h/compile p/output!))

#?(:clj (t/ann native-service [-> nil]))
(defn native-service []
  (let [directory (p/input!)
        probe (h/probe "/bin/test" ["-f" "service.ready"] 50 1000)
        service (-> (h/task "service" "/bin/sh" ".")
                    (h/with-args ["-c" "echo $$ > service.pid; touch service.ready; exec sleep 120"])
                    (h/as-service (-> (h/service probe) (h/with-health probe) (h/with-service-limits 10000 5000 2)))
                    (h/with-shared-resources ["checkout"]))
        client (-> (h/task "client" "/bin/sh" ".")
                   (h/with-args ["-c" "test -f service.ready && printf healthy"])
                   (h/with-shared-resources ["checkout"]) (h/after [service]))]
    (finish! (-> (h/plan [service client]) (h/with-jobs 1) (h/with-max-services 1)
                 (h/with-resource-directory "leases")) directory "native service complete")))
