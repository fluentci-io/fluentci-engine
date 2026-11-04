# FluentCI plugin SDKs

Language SDKs write Wasm guests that call the same PDK functions exposed by
`fluentci-engine` in the `extism:host/user` namespace. The engine supervises the
native processes; plugins do not embed an Extism host runtime.

| Language | Build path | Guide |
| --- | --- | --- |
| Python | uv project → extism-py → Wasm | [Python PDK](python/README.md) |
| Elixir | supported CI DSL → Rust PDK → Wasm | [Elixir DSL](elixir/README.md) |
| Gleam | Gleam → JavaScript → extism-js → Wasm | [Gleam PDK](gleam/README.md) |
| PureScript | PureScript → JavaScript → extism-js → Wasm | [PureScript](purescript/demo-plugin/README.md) |
| ReScript | ReScript 12 → JavaScript → extism-js → Wasm | [ReScript PDK](rescript/README.md) |
| TypeScript | esbuild → extism-js → Wasm | [TypeScript](typescript/demo-plugin/README.md) |

The immutable builders produce reusable values; terminal operations invoke the
engine. `exec` preserves literal arguments in these APIs; `shell` explicitly opts
into shell syntax. Elixir supports the documented DSL subset, not arbitrary BEAM
applications. Guest runtime dependencies must fit their Wasm sandbox.

The plugin PDK CI workflow builds each language and invokes the compiled Wasm
through the release engine. Run `python3 scripts/xtp_codegen.py --check` to verify
that guest ABI declarations match the canonical schema.

## Parallel matrices

All six language examples expose native matrix execution. Case construction is
pure; one `execute_plan` PDK call runs the complete plan. See the
[matrix guide](../docs/matrix.md) for products, typed include/exclude, stable IDs,
prerequisites, joins, caching and cancellation. Requires FluentCI Engine 0.6.1 or
later and matching SDKs.

## Existing host APIs

# SDK development

Examples should compile against this checkout, or an explicit immutable SDK Git
revision when the library is maintained in another repository. Commit lockfiles
when changing dependency declarations.

| SDK | Execution APIs | Local validation |
| --- | --- | --- |
| Elixir | Immutable host plans, local execution, typed results/journals | Elixir 1.20.4: `mix format --check-formatted`; `FLUENTCI_REQUIRE_ENGINE=1 mix test --warnings-as-errors` |
| Gleam | Host plans, local/HTTP execution, results/journals; legacy bindings | `gleam test --runtime nodejs`; `gleam test --runtime deno`; `cd demo && gleam build` |
| PureScript | External pinned SDK with host plans, local/HTTP execution and legacy bindings | Spago 1.0.5/PureScript 0.15.16: `spago build` in each example; `python3 tests/e2e_purescript.py` from repository root |
| ReScript | Immutable guest PDK, legacy GraphQL/plugin/Dagger bindings | ReScript 12.3.1: `npm ci && npm run res:build` in SDK and demo |
| TypeScript | Legacy GraphQL, plugin, and Dagger bindings | Deno 2: `deno task check`; plugin JavaScript bundles: `cd demo-ext && bun install --frozen-lockfile && node esbuild.js` |

Host API coverage is not yet equivalent across languages: TypeScript and ReScript
host-plan adapters remain pending. Compilation does not execute legacy examples;
those require a compatible engine session. Plugin JavaScript bundling does not
validate final Wasm execution, which additionally requires Extism tooling.

Build the test engine with `cargo build --release --locked -p fluentci-engine`.
`FLUENTCI_ENGINE_BIN` can select a different release binary for integration tests.
The host-executor workflow checks examples and SDK contracts on Ubuntu, including
PureScript's saved plan against the actual release engine.
