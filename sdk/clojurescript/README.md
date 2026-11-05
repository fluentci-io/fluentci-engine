# ClojureScript SDK

Immutable pipelines, literal argument vectors, ordinary function composition and
thread-first (`->`) chaining. Persistent tagged maps distinguish `Pipeline`,
`File` and `Secret`. Builders perform no I/O; terminal functions end in `!`.

```clojure
(ns my.plugin
  (:require [fluentci.plugin :as p]
            #?(:clj [typed.clojure :as t])))

#?(:clj (t/ann release [p/Pipeline -> p/Pipeline]))
(defn release [pipeline] (p/env pipeline "MODE" "release"))

#?(:clj (t/ann build [t/Str -> t/Str]))
(defn build [directory]
  (-> (p/pipeline "build")
      release
      (p/workdir directory)
      (p/cache "cargo" "target")
      (p/exec ["cargo" "build" "--release"])
      p/stdout!))
```

## Static typing

Write checked code in `.cljc`. The mandatory build first checks the **JVM view of
the shared source** with Typed Clojure 1.4.0, then compiles its ClojureScript view
using ClojureScript 1.12.145. All SDK and example functions have explicit static
signatures. Four negative fixtures prove that non-string argv, passing a `File`
to `stdout!`, incorrect transformation results and chaining after a terminal
operation are rejected. These checks run before JavaScript or Wasm compilation.

This is not a general static type checker for arbitrary `.cljs` code. Keep logic
in shared `.cljc`, without platform-specific branches. Only `fluentci.bridge`
(Extism globals/JSON) and the tiny CommonJS export shim use unchecked JavaScript
interop. Their declared signatures are trusted; compiled JS contract tests and
actual Wasm engine tests verify that boundary. Annotations are erased from guest
code. There is no Typed Clojure runtime inside Wasm.

Consumer projects should include their namespaces in `clojure -M:check`, as the
[ported hash plugin](../../examples/hash-clojurescript/README.md) does. The checker
alias sets `typed.clojure.preserve-check-ns-after-opt-in=true` so checks cannot
silently skip namespaces. Adding arbitrary `.cljs` code bypasses this model.

## Setup and build

Requires JDK 21+, Node.js, mise, extism-js (CI pins 1.7.0), Binaryen and the
FluentCI engine. Clojure CLI **1.12.5.1654** is pinned in `mise.toml`; Maven
dependencies are pinned in `deps.edn`, and npm dependencies in `package-lock.json`.

```sh
cd sdk/clojurescript
mise trust
mise install clojure
mise exec -- clojure -M:check
mise exec -- clojure -M:test
node build/tests.cjs
cd demo-plugin
npm ci
mise exec -- npm run build
fluentci-engine call -m dist/plugin.wasm greet 'hello from ClojureScript'
```

The build is shared source → static check → JavaScript → esbuild CommonJS →
extism-js → `demo-plugin/dist/plugin.wasm`. The guest imports engine PDK functions
through `Host` and `Memory`; it embeds no Node, HTTP client or Extism host runtime.
`imports.d.ts` is generated from `schema/host.json`.

## API

| Pure operation | Signature |
| --- | --- |
| `pipeline` | `Str -> Pipeline` |
| `exec` | `Pipeline, Vec<Str> -> Pipeline` |
| `shell`, `workdir` | `Pipeline, Str -> Pipeline` |
| `env`, `cache` | `Pipeline, Str, Str -> Pipeline` |
| `file` | `Str -> File` |
| `secret` | `Str, Str -> Secret` |
| `with-secret` | `Pipeline, Str, Secret -> Pipeline` |
| `copy-file` | `Pipeline, File, Str -> Pipeline` |

`stdout!`/`stderr!` execute a pipeline; `sha256!`/`md5!` hash a file;
`call-module!` calls a nested plugin. All return strings. `input!` reads the guest
request, and `output!` writes a string response. Host failures throw `ex-info`
with `:kind :host-error` and the PDK `:function`; process exit codes propagate
through the engine. Empty argv and NUL-containing arguments fail at construction.
`exec` preserves literal arguments, including quotes, spaces and shell metacharacters;
`shell` explicitly opts into shell syntax. Unix commands/quoting are used.

This initial SDK covers guest pipelines, caches, secrets, file hashing/copying and
nested calls. Durable host plans, matrices, providers and services are not yet
exposed. See [demo-plugin](demo-plugin/README.md) and the complete
[hash port](../../examples/hash-clojurescript/README.md).
