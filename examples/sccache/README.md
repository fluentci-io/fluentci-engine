# Rust builds with sccache, written in PureScript

All runtime logic lives in [src/Sccache.purs](src/Sccache.purs), using the
[PureScript FluentCI SDK](https://github.com/fluentci-io/purescript-fluentci).
Platform selection, pinned release metadata, checksum verification, provisioning,
plan composition and error handling are PureScript. Native commands receive
literal arguments; there are no shell scripts, shell commands or custom JS FFI.
`entry.mjs` only exposes the PureScript export, and `build.mjs` invokes the compilers.

On a build host with PureScript 0.15.16, Spago 1.0.5, Node and extism-js 1.7.0:

```sh
cd examples/sccache
npm ci
npm run build
npm test
cd ../..
fluentci-engine call -m examples/sccache/dist/plugin.wasm build \
  '["fixtures/sccache-demo", "runs/rust-first"]'
```

The first two JSON array values select the project and a **fresh** run directory.
Additional values are literal Cargo build arguments, for example
`["my project", "runs/build-2", "--package", "my-library", "--features", "fast"]`.
The plugin always uses `cargo build --release --locked` and rejects profile
overrides. It requires a matching source build of the engine with `execute_plan`.

The functions `artifactFor`, `validateArguments`, `downloadPlan`, `installPlan`
and `buildPlan` are pure and composable. Effects are explicit at `provision` and
`buildProject`. The build plan starts sccache directly as a foreground managed
service, waits for its private Unix socket, builds the project, collects JSON
statistics and shuts the service down. The engine handles cancellation and reaps
its processes. An exclusive storage lease prevents two builds managed by this
example from starting servers against the same cache simultaneously.

An existing sccache on PATH is reused. Otherwise PureScript selects a pinned
0.18.0 GitHub release for Linux/macOS on x86_64/arm64, downloads it with native OS
curl, verifies its SHA-256 using the engine's PDK, then extracts it with native
tar and atomically moves the executable into place. No sudo or profile changes
are required. See the [upstream releases](https://github.com/mozilla/sccache/releases).

`SCCACHE_INSTALL_DIR` defaults to `$HOME/.local/bin`; `SCCACHE_DIR` defaults to
`$HOME/.cache/sccache`. Both settings must be absolute. The build plan sets
`RUSTC_WRAPPER`, `CARGO_INCREMENTAL=0` and a 1 GiB cache budget while inheriting
host Rustup/Cargo configuration. An unrelated sccache server does not participate
in the executor's leases; dedicate the selected cache directory to this example.

The plugin returns the native `RunResult` JSON report, with logs, statistics and
events retained in the requested run directory. Provisioning and cleanup reports
use adjacent directories with `.probe`, `.stage`, `.download`, `.install` and
cleanup suffixes. Failures propagate to the plugin error boundary. Temporary
installer files and the socket are removed after ordinary success/failure;
interruption can leave staging files or an inactive socket for manual cleanup.

Compiler caching is separate from FluentCI's action cache. Rebuild the library
with a different `CARGO_TARGET_DIR` and a fresh run directory to observe a hit;
otherwise Cargo may skip rustc entirely. Final executable linking is not cached
by sccache. See the [Rust caching notes](https://github.com/mozilla/sccache#rust).

CI compiles this PureScript plugin to Wasm, tests its pure plan constructors,
then runs `tests/e2e_sccache.py` on a disposable Linux runner. That test provisions
the binary, verifies cold misses and warm hits across fresh Cargo target
directories, and checks managed-service shutdown and socket cleanup.
