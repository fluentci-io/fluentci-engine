# Hash plugin in ClojureScript

A complete port of [examples/hash](../hash/src/lib.rs), using the local
[ClojureScript SDK](../../sdk/clojurescript/README.md). Both `md5` and `sha256`
accept a file path as raw guest input and return the hexadecimal digest. The
engine performs hashing through the same native PDK calls as the Rust plugin.

The plugin logic is in [src/hash/plugin.cljc](src/hash/plugin.cljc). Each export
is an idiomatic thread-first chain: input → file → hash → output. Typed Clojure
checks that shared source before ClojureScript compilation. JavaScript contains
only the export shim; there is no hashing implementation or custom SDK FFI here.

With JDK 21+, Node, mise, extism-js, Binaryen and FluentCI Engine installed:

```sh
cd examples/hash-clojurescript
mise trust
mise install clojure
npm ci
mise exec -- npm run build
fluentci-engine call -m dist/plugin.wasm md5 README.md
fluentci-engine call -m dist/plugin.wasm sha256 README.md
```

`mise.toml` pins Clojure CLI 1.12.5.1654. The build requires static checking,
compiles shared `.cljc` to JavaScript, bundles CommonJS, then compiles
`dist/plugin.wasm`. `imports.d.ts` is generated from the canonical engine schema.
The type checker checks the JVM view of the shared code; the SDK's JavaScript
guest bridge is a trusted, integration-tested boundary. See the SDK guide for
the limits of this typing model.

CI builds this plugin and verifies both digests through the actual engine for
empty, binary and Unicode file contents, paths with spaces/quotes, and missing
files. Run those checks from the repository root after building the engine:

```sh
python3 tests/e2e_guest_pdks.py ClojureScriptHashPlugin -v
```
