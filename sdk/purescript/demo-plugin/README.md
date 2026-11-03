# Purescript Wasm plugins

Use `FluentCI.Plugin` to call the PDK functions exposed by FluentCI from inside a Wasm guest.

```purescript
P.pipeline "greet" # P.exec ["/usr/bin/printf", "%s", "hello"] # P.stdout
```

Install PureScript 0.15.16 and Spago 1.0.5, Node.js, and [extism-js v1.7.0](https://github.com/extism/js-pdk).

```sh
cd sdk/purescript/demo-plugin
npm ci
npm run build
fluentci-engine call -m dist/plugin.wasm greet 'hello from purescript'
```

The build compiles to JavaScript, bundles ES2020 CommonJS, and compiles an Extism
Wasm guest. It calls `Host`/`Memory` guest globals, without embedding a host runtime
or using an HTTP client. Dependencies must fit the Extism JavaScript sandbox;
Node/Deno OS APIs are unavailable inside Wasm. `imports.d.ts` comes from the
canonical `schema/host.json`.

The example pins the refactored SDK's immutable `FluentCI.Plugin` API. Errors are
`PluginError` values; use `errorMessage` when displaying them. Guest input/output
also use explicit typed effects.

Additional exports exercise failure diagnostics (`failure`, `invalid`), file
hashing/copying (`hash_file`, `copy_file`), nested plugins (`nested`), secret injection
(`secret_env`), supervised services (`service`, `cancel`), and pure host-plan
construction (`action_plan`). Engine CI runs these against the compiled Wasm;
`action_plan` is executed separately to verify action-cache restore/invalidation.
