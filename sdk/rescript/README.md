# FluentCI Rescript SDK

This is the official [FluentCI](https://fluentci.io) SDK for Rescript. It allows you to write CI/CD pipelines in Rescript.

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
fluentci-engine run -- src/Demo.res.mjs  
```

See [fluentci-engine](https://github.com/fluentc-io/fluentci-engine) and [fluentci](https://github.com/fluentci-io/fluentci) for more information.
