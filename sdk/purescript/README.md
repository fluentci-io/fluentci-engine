# PureScript SDK examples

The SDK library lives in [purescript-fluentci](https://github.com/fluentci-io/purescript-fluentci).
These examples pin commit `324a638224654b0df38f912b9e3e6f73485e4021`, including
immutable host plans, local/HTTP execution, services, journals, and verified
`cache.toolchain_files` support.

Use PureScript 0.15.16, Spago 1.0.5, and Node.js 26:

```sh
npm install --global purescript@0.15.16 spago@1.0.5
cd sdk/purescript/demo-host
spago build
node --input-type=module -e 'import { main } from "./output/Main/index.js"; main()' > plan.json
printf 'hello\n' > input
fluentci-engine execute plan.json --run-dir run --progress plain
```

`demo-host` builds a plan using `#` and performs no I/O until its `main` prints
JSON. The engine executes the plan and verifies the declared `/bin/cat` toolchain
file. The example uses Unix paths. Run a second time with a fresh run directory to
observe a cache hit. Keep inputs and the cache directory between invocations.

`demo` demonstrates legacy GraphQL pipeline builders. Configuration arguments come
first; effectful builders compose with `>>=`, while value transformations use `#`.
`demo-dagger` demonstrates the separate container-based Dagger bindings. Both
require the JavaScript `@fluentci/sdk` runtime and their respective engine sessions.
They are compiled here, but their engines are not started during compilation.

Each directory uses Spago YAML and a committed lockfile. To update the SDK, change
the Git ref in each `spago.yaml`, regenerate the locks, and rebuild all examples.
