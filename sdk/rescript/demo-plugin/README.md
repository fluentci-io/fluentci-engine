# Rescript Wasm plugins

Use `Guest` to call the PDK functions exposed by FluentCI from inside a Wasm guest.

```rescript
Guest.pipeline("greet")->Guest.exec(["/usr/bin/printf", "%s", "hello"])->Guest.stdout
```

Install ReScript 12.3.1, Node.js, and [extism-js v1.7.0](https://github.com/extism/js-pdk).

```sh
cd sdk/rescript/demo-plugin
npm ci
npm run build
fluentci-engine call -m dist/plugin.wasm greet 'hello from rescript'
```

The build compiles to JavaScript, bundles ES2020 CommonJS, and compiles an Extism
Wasm guest. It calls `Host`/`Memory` guest globals, without embedding a host runtime
or using an HTTP client. Dependencies must fit the Extism JavaScript sandbox;
Node/Deno OS APIs are unavailable inside Wasm. `imports.d.ts` comes from the
canonical `schema/host.json`.

Build the SDK first: `cd sdk/rescript && npm ci && npm run res:build`.
