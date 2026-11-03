# Rust builds with sccache

This Wasm plugin runs a native executor plan: provision sccache, start its server,
run `cargo build --release --locked`, collect JSON statistics, then stop the server.
The server is a managed service: readiness, cancellation and process cleanup use
FluentCI's executor. A private Unix socket isolates it from other runner daemons.
An exclusive storage lease serializes builds sharing the same compiler cache.

Build the plugin, then call it with a project and a **fresh** run directory:

```sh
cargo build --release --locked --target wasm32-unknown-unknown -p sccache
fluentci-engine call -m target/wasm32-unknown-unknown/release/sccache.wasm build \
  '["fixtures/sccache-demo", "runs/rust-first"]'
```

Additional JSON array elements are literal Cargo build arguments, for example
`["my project", "runs/build-2", "--package", "my-library", "--features", "fast"]`.
The release profile and lockfile are required; profile overrides are rejected.
This example requires a matching source build of the engine with `execute_plan`.

The output is a native `RunResult` JSON report. The `stats` task's `stdout` file
contains sccache's JSON counters. The run directory retains logs and events even
when a build fails; failures propagate through the plugin error boundary.

An existing `sccache` on PATH is reused. Otherwise the plugin downloads the pinned
0.18.0 GitHub release, verifies its SHA-256 checksum and installs it atomically in
`SCCACHE_INSTALL_DIR` (default `$HOME/.local/bin`). Provisioning supports Linux and
macOS on x86_64 and arm64, using native OS curl and tar without sudo or profile
changes. See the [upstream releases](https://github.com/mozilla/sccache/releases).

`SCCACHE_DIR` selects the shared compiler cache (default `$HOME/.cache/sccache`);
both directory settings must be absolute. The example sets `RUSTC_WRAPPER=sccache`,
`CARGO_INCREMENTAL=0` and a 1 GiB compiler-cache budget. Other host environment
settings, including Rustup configuration and `CARGO_TARGET_DIR`, are inherited.
Only share this storage with builds managed by this example, since an unrelated
sccache server does not participate in the executor's resource leases.

Compiler caching is separate from FluentCI's action cache. To observe a hit,
rebuild the library with a different `CARGO_TARGET_DIR` and a fresh run directory.
Without cleaning or changing the target directory, Cargo may skip rustc entirely.
Rust library compilation can be cached; final executable linking is not cached
by sccache. See the [Rust caching notes](https://github.com/mozilla/sccache#rust).

`tests/e2e_sccache.py` provisions the real binary on disposable GitHub runners,
builds the standalone fixture twice with fresh target directories, checks cold
misses and warm hits, and verifies server/socket cleanup.
