# PureScript CI modules

The root `fluentci.toml` declares this repository's Wasm modules. All pipeline
orchestration uses the PureScript SDK. The former Dagger module, vendored Dagger
SDK, TypeScript runner, Deno manifests, and nested `dagger.json` are removed.
Product SDKs under `sdk/` are independent and remain part of the engine tests.

```sh
fluentci modules
fluentci run rust test
fluentci run rust build
fluentci run e2e check
fluentci run main check
```

| Module | Functions | Purpose |
| --- | --- | --- |
| `main` | `check`, `test`, `build`, `e2e` | Default CI and convenient entry points |
| `rust` | `test`, `build`, `clippy`, `format`, `llvmCov` | Native release-mode Rust tasks |
| `e2e` | `check`, `buildWasm`, `providers`, `typescript`, `graphql` | Existing 13 provider plugins, TypeScript SDK demo, and 17 GraphQL fixtures |
| `release` | `build`, `package`, `upload`, `r2` | Native Linux/macOS builds, checksums, GitHub and R2 uploads |
| `nix` | `check`, `metadata`, `flakehub`, `flakestry` | Explicit Nix validation and publication |

`common` is a shared PureScript source package. Spago path dependencies compose
modules without nested CLI calls. Every Cargo build, test, lint, and coverage
command uses release mode and the lockfile. The native release target determines
which runner is used; Linux releases no longer use cross linkers inside Dagger.

## Toolchains and workflow bootstrap

Every GitHub Actions workflow uses `fluentci-io/setup-fluentci@next` with the
published bootstrap CLI version `2026.11.04`. The module-building jobs use the
`.github/actions/fluentci-modules` composite: Node 24, PureScript 0.15.16, Spago
1.0.5, Extism JS 1.7.0, and Binaryen 133. The shared script verifies archive
checksums and probes all tools, including `wasm-merge` and `wasm-opt`. It also
runs six module integration tests with mocked build and upload commands, checking
release arguments, provider coverage, packaging, and failure propagation without
publishing assets.

Linux/macOS release workflows compile the release module once on Ubuntu, transfer
its Wasm file as an artifact, then run it on the native target runner:

```sh
TARGET=aarch64-apple-darwin \
  fluentci run --wasm .fluentci/release/dist/plugin.wasm build
```

Native builders need no PureScript, Node, or Extism compiler. The existing BSD,
SDK/PDK, host-executor, workflow-primitives, package-provider, and trap validation
jobs retain their native toolchains and coverage; they also install the new CLI
through the setup action. Nix publication remains explicitly triggered by its
existing workflows. JSR SDK publication runs after successful push CI.

## Provider end-to-end tests

`e2e check` builds the engine and provider Wasm plugins, exercises each provider,
runs the TypeScript SDK consumer through the engine, and checks the GraphQL
fixtures against a managed engine service. These tests install or invoke real
package-provider tools and can require the existing provider credentials and
network access. The CI workflow retains its native Flox setup and scoped secrets.
The TypeScript consumer needs Deno; CI installs it explicitly rather than relying
on the old setup action's side effects.

## Releases

Set `TARGET` and `RELEASE_VERSION` before `release package` or `release upload`.
The archive contains `fluentci-engine` at its root, with a `.sha256` sidecar.
GitHub uploads use `GH_TOKEN`; `release r2` retains `ACCOUNT_ID` and the existing
AWS credential environment for Cloudflare R2. Uploads are not part of `main check`.
Native BSD packaging remains in the existing verified BSD release workflow.
