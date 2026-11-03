# Gleam Wasm plugins

Use `fluentci/plugin` to call the PDK functions exposed by FluentCI from inside a Wasm guest.

```gleam
plugin.pipeline("greet") |> plugin.exec("/usr/bin/printf", ["%s", input]) |> plugin.stdout
```

Install Gleam 1.19.2, Node.js, and [extism-js v1.7.0](https://github.com/extism/js-pdk).

```sh
cd sdk/gleam/demo-plugin
npm ci
npm run build
fluentci-engine call -m dist/plugin.wasm greet 'hello from gleam'
```

The build compiles to JavaScript, bundles ES2020 CommonJS, and compiles an Extism
Wasm guest. It calls `Host`/`Memory` guest globals, without embedding a host runtime
or using an HTTP client. Dependencies must fit the Extism JavaScript sandbox;
Node/Deno OS APIs are unavailable inside Wasm. `imports.d.ts` comes from the
canonical `schema/host.json`.
