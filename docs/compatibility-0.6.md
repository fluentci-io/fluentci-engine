# FluentCI Engine 0.6 compatibility

Version 0.6.0 unifies execution under the host plan executor and adds guest plugin
build paths for Python, Elixir, Gleam, PureScript, ReScript, and TypeScript.
Legacy APIs remain compatibility entry points. This is a minor engine release;
Rust library crates and language SDKs retain their independent version numbers.
Version 0.6.1 adds native matrices, workflow policies, file/env-file primitives,
and process failure diagnostics; use matching SDKs for these additions.
Version 0.6.2 adds composable SSH/transfer/S3, host facts and digest tasks, typed
compiled plans, compact dependency graphs, and incremental scheduling for output
publication and allowed failures. See the executor benchmark report for measured
graph-construction improvements; command execution gains are workload-dependent.

## Execution and scheduling

Legacy GraphQL/Wasm graphs, eager operations, provider setup, cache finalization,
and service lifecycles compile into executor-managed tasks. The executor owns
failure propagation, process-group cancellation, log collection, and cleanup.
Built-in provider hooks run in isolated `provider-worker` subprocesses with
private request files and explicit environments.

| Boundary | Compatibility behavior | Migration path |
| --- | --- | --- |
| Task ordering | Legacy plans retain serial command ordering. Sharing an executor does not make an existing pipeline parallel. | Use explicit tasks, `needs`, and `jobs` in a host plan to declare safe parallelism. |
| Graph state | The legacy GraphQL API retains mutable current-pipeline state. New guest builders lower values into that API when invoked. | Use explicit independent host plans for concurrent local/server runs; do not share a legacy graph between independent clients. |
| Services | Legacy service APIs preserve a five-second startup grace and process-liveness supervision. They cannot declare protocol readiness probes. Services are supervised by the executor instead of process-compose. | Declare explicit service readiness probes for databases and other long-running dependencies. Avoid depending on generated process-compose files. |
| Run history | Compatibility request/run directories are temporary. Sharing the executor does not automatically publish compatibility runs to durable history. | Use `execute` or `serve-runs` for retained journals/results and the run API. `serve-runs` remains separate from GraphQL `serve`. |
| Custom extensions | Built-in providers supply serializable worker descriptors. A custom extension without `worker_spec` cannot use an in-process execution fallback. | Implement a compatible descriptor/worker integration before routing the extension through the compatibility compiler. |
| Embedding | Provider workers require the matching engine executable. | Library embedders must set `Graph.worker_executable`; the engine CLI selects its own binary. |

## Caching and host isolation

Legacy cache mounts and cleanup are not the same contract as the explicit
content-addressed action cache. Action caching is opt-in: declare inputs, outputs,
and environment values. Cached tasks require `inherit_env: false`. Relocatable
cache reuse requires the caller to guarantee path-independent output and logs.
The explicit action cache is local; remote cache storage and retention/garbage
collection remain follow-up work. Existing provider caches do not imply equivalent
remote action-cache support.

Workspace copies separate run files but do not sandbox host access. Native
processes use host permissions, installed tools, and available network access.
Declare shared resource names for cooperating tasks/runs that access a package
prefix or other shared mutable state. Cancellation and graceful shutdown clean up
supervised processes; machine crashes and hard kills cannot guarantee cleanup on
every supported OS. Interrupted server runs are reconciled, not automatically resumed.

## Guest SDKs and Wasm

All guest APIs call the PDK functions exposed by the FluentCI engine through
`extism:host/user`. They do not embed Extism host SDKs. The canonical import ABI is
`schema/host.json`; generated declarations must match it. See the
[SDK guides](../sdk/README.md) for build commands and API examples.

| Language | Supported build path and limits |
| --- | --- |
| Python | uv manages the typed SDK project; `fluentci-plugin` prepares canonical imports and invokes extism-py. The guest embeds CPython 3.12 and supports pure Python dependencies, not native CPython extensions. The PyPI `extism` host package is not the guest PDK. |
| Elixir | The documented immutable CI DSL generates Rust PDK source and compiles it to Wasm. This is a supported subset, not arbitrary Elixir/BEAM or OTP compilation. Elixir evaluates declarations at build time; generated PDK operations execute in Wasm. |
| Gleam | Compile the JavaScript target, bundle with esbuild, then compile with extism-js. The plugin runs in the Extism JavaScript sandbox, without Node/Deno OS APIs. |
| PureScript | Compile to JavaScript, bundle, then compile with extism-js. The guest SDK is maintained in the sibling repository and examples pin an immutable Git revision. |
| ReScript | ReScript 12.3.1 replaces ReScript 11 and the separate `@rescript/core` dependency. Use the bundled standard library, `rescript watch`, and the updated configuration keys. Compile JavaScript through esbuild/extism-js. |
| TypeScript | Bundle the synchronous guest PDK with esbuild, then compile with extism-js. Existing asynchronous GraphQL clients remain host/session APIs and cannot replace guest bindings inside Wasm. |

Guest and host-plan APIs are separate entry points and do not yet offer identical
features across every language. Immutable guest `exec` helpers preserve literal
argument boundaries; explicit `shell` operations accept shell syntax. Legacy
`withExec` retains its existing shell-fragment semantics, with arguments encoded
as a JSON array on the host ABI. Do not substitute legacy calls for literal-argv
helpers when passing untrusted text.

The NetBSD Wasmtime adjustment is a workspace patch. Cargo does not propagate a
workspace's `[patch.crates-io]` overrides to downstream consumers. Use the source
checkout or platform release artifacts when you need the patched BSD runtime;
installing published library crates alone does not carry that workspace override.

## Release preparation

The source engine, Nix package, and README download links target 0.6.2. Workflow
bootstrap pins independently select the published 0.5.0 engine.
Bumping source versions does not tag a release, publish crates, or establish that
all platform CI jobs have passed. Build and smoke-test release artifacts on the
supported platform matrix before publishing them.
