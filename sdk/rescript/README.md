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

## Existing APIs

# FluentCI ReScript SDK

This is the official [FluentCI](https://fluentci.io) SDK for ReScript. It allows you to write CI/CD pipelines in ReScript.

## Installation

```sh
bun add rescript-fluentci
```

## Quick Start

```rescript
open FluentCI
open FluentCI.Deno

let output =
  await dag
  ->Client.pipeline(~name="demo")
  ->Pipeline.withExec(["echo", "Hello World!"])
  ->Pipeline.stdout

Console.log(output)
```

Run the following command to execute the pipeline:

```sh
bun run res:build
fluentci-engine run -- deno run -A src/Demo.res.mjs  
```

See [fluentci-engine](https://github.com/fluentci-io/fluentci-engine) and [fluentci](https://github.com/fluentci-io/fluentci) for more information.

## Development

Use Node.js 20.11 or newer. Run `npm ci && npm run res:build` here, then
run the same commands in `demo`. The demo uses `file:..` so it compiles against this checkout. These are
legacy GraphQL/plugin/Dagger bindings; immutable host-plan bindings are still
pending for ReScript. Compilation does not start an engine or execute pipelines.

The SDK and demo use ReScript 12.3.1 with its bundled standard library.
Use `npm run res:dev` for watch mode and `npm run res:format` to format sources.
The checked-in npm lockfiles pin the compiler and JavaScript dependencies.
