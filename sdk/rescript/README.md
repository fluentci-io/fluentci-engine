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
