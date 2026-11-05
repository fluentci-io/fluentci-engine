# ClojureScript Wasm guest

See the [SDK setup and type-checking model](../README.md). From this directory:

```sh
npm ci
mise exec -- npm run build
fluentci-engine call -m dist/plugin.wasm greet 'héllo 世界'
```

`demo.plugin` contains statically checked shared `.cljc` logic. `release` is an
ordinary reusable pipeline transformation; `greeting` chains it with `->`.
Exports: `greet`, `echo`, `failure`, `hash_file`, `copy_file`, `nested`.
`nested` expects a copy of this guest named `plugin.wasm` in the caller's workdir.

`npm run build` requires static checking to pass before compiling JavaScript and
Wasm. CI invokes the compiled guest through the real engine, including literal
argv, nonzero process exits, quoted/Unicode file paths and nested requests.
