# FluentCI Elixir plugin SDK

Write immutable CI declarations and compile them into Extism Wasm plugins that
call FluentCI's exposed PDK functions. No Extism host SDK or NIF is embedded.

```elixir
defmodule DemoPlugin do
  use FluentCI.Plugin

  defplugin greet do
    pipeline("greet")
    |> exec(["/usr/bin/printf", "%s", input()])
    |> stdout()
  end
end
```

Add `{:fluentci, path: "../fluentci-engine/sdk/elixir"}` to Mix dependencies.
Compose `workdir`, `env`, `cache`, `exec`, and explicit `shell` operations with `|>`.
`exec` preserves literal argument boundaries. `input()` resolves to the invocation
input inside Wasm. `stdout`/`stderr` select the return value.

## Compile and run

Requires Elixir 1.20.4, Rust 1.99.1, and the `wasm32-unknown-unknown` Rust target.

```sh
cd sdk/elixir/demo-plugin
mix deps.get
mix fluentci.plugin.build DemoPlugin --output dist/plugin.wasm
fluentci-engine call -m dist/plugin.wasm greet 'hello from Elixir'
```

Use `--pdk-path ../../../crates/pdk` to build against this checkout. The task
retains generated Rust source under `.fluentci/plugins/`, locks dependencies, and
runs `cargo build --release --locked`.

This is a supported CI DSL subset, not general Elixir/BEAM-to-Wasm compilation.
Elixir evaluates declarations at build time; only generated PDK operations run
inside Wasm. Arbitrary Elixir functions and OTP processes cannot run in the guest.
Exports return a pipeline, `input()`, or a string. Code outside the DSL has normal
Elixir build-time effects. The separate [host-plan adapter](HOST_PLANS.md) remains
available for JSON plan clients.

## Parallel matrix jobs

See the [matrix guide](../../docs/matrix.md) and the `matrix` export in this SDK's
plugin example. The API expands named typed cases into native tasks and submits
one complete plan via `execute_plan`. It supports bounded concurrency, independent
case caching/logs, prerequisites, joins, failure reports and owning-run cancellation.
Requires FluentCI Engine 0.6.1 or later and matching SDKs.
