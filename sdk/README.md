# SDK development

Examples should compile against this checkout, or an explicit immutable SDK Git
revision when the library is maintained in another repository. Commit lockfiles
when changing dependency declarations.

| SDK | Execution APIs | Local validation |
| --- | --- | --- |
| Elixir | Immutable host plans, local execution, typed results/journals | Elixir 1.20.4: `mix format --check-formatted`; `FLUENTCI_REQUIRE_ENGINE=1 mix test --warnings-as-errors` |
| Gleam | Host plans, local/HTTP execution, results/journals; legacy bindings | `gleam test --runtime nodejs`; `gleam test --runtime deno`; `cd demo && gleam build` |
| PureScript | External pinned SDK with host plans, local/HTTP execution and legacy bindings | Spago 1.0.5/PureScript 0.15.16: `spago build` in each example; `python3 tests/e2e_purescript.py` from repository root |
| ReScript | Legacy GraphQL, plugin, and Dagger bindings | `bun install --frozen-lockfile && bun run res:build` in SDK and demo |
| TypeScript | Legacy GraphQL, plugin, and Dagger bindings | Deno 2: `deno task check`; plugin JavaScript bundles: `cd demo-ext && bun install --frozen-lockfile && node esbuild.js` |

Host API coverage is not yet equivalent across languages: TypeScript and ReScript
host-plan adapters remain pending. Compilation does not execute legacy examples;
those require a compatible engine session. Plugin JavaScript bundling does not
validate final Wasm execution, which additionally requires Extism tooling.

Build the test engine with `cargo build --release --locked -p fluentci-engine`.
`FLUENTCI_ENGINE_BIN` can select a different release binary for integration tests.
The host-executor workflow checks examples and SDK contracts on Ubuntu, including
PureScript's saved plan against the actual release engine.
