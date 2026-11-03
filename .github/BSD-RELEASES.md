# Native BSD release archives

The `BSD releases` workflow builds the **full CLI** on native x86_64 guests:

| Target | Build guest | Toolchain | Release policy |
| --- | --- | --- | --- |
| `x86_64-unknown-freebsd` | FreeBSD 14.3 | Official Rust 1.99.1 | Build, smoke-test, package |
| `x86_64-unknown-netbsd` | NetBSD 10.1 | Official Rust 1.99.1 | Build with the vendored Wasmtime signal-context fix, smoke-test, package |
| `x86_64-unknown-openbsd` | OpenBSD 7.8 | Native package must match Rust 1.99.1 | Explicitly skip while unavailable; never substitute an older compiler |

The compiler pin is read from `rust-toolchain.toml`. OpenBSD does not have
upstream precompiled Rust host tools; its pinned 7.8 package repository currently
provides Rust 1.90.0. Its workflow reports the mismatch in the job summary and
uploads no OpenBSD asset. A matching native package enables that build without
changing the compiler requirement. Other build or smoke-test failures fail the
job; they are not silently classified as unsupported platforms.

Only x86_64 is covered. These are dynamically linked native binaries. Use them on
the documented guest OS/version and architecture, with the shared libraries
listed in the archive's `BUILD-INFO.txt`. OpenBSD does not promise ABI compatibility
between OS releases. No container or VM is required to run the installed engine.

## Assets

The naming matches existing Linux/macOS releases:

```text
fluentci-engine_<tag>_x86_64-unknown-freebsd.tar.gz
fluentci-engine_<tag>_x86_64-unknown-freebsd.tar.gz.sha256
fluentci-engine_<tag>_x86_64-unknown-netbsd.tar.gz
fluentci-engine_<tag>_x86_64-unknown-netbsd.tar.gz.sha256
```

The OpenBSD pair uses the same convention when its compiler gate passes.
Each archive contains the executable, build/source/toolchain metadata, project
license, this guide, and the vendored SuperConsole/Wasmtime licenses.
Compare `sha256 -q <archive>` against its `.sha256` file before extracting.
Install the extracted executable into a directory on `PATH`, for example:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 fluentci-engine "$HOME/.local/bin/fluentci-engine"
"$HOME/.local/bin/fluentci-engine" --version
```

## Build, verification, and publishing

Every Cargo build uses `--release --locked`. FreeBSD/NetBSD download the exact
Rust host-tool archive over HTTPS with the native OS curl and verify its official
SHA-256 sidecar before installation. Native toolchains, Cargo artifacts/downloads,
and the sccache disk backend are cached separately by OS version and target.
Builds do not reuse Linux compiler executables inside BSD guests.

Before packaging the stripped binary, each supported guest must pass:

- CLI startup and help;
- XTP-generated Wasm exports and host environment calls;
- actual `unreachable` and dynamically computed out-of-bounds memory traps;
- a native host pipeline and a second run restoring its output from cache;
- dynamic-library resolution.

Copied-back archives are checked again on the Linux runner: checksum, safe member
paths, executable permissions, x86_64 ELF header, required files, and matching
build metadata. Only verified archives become workflow artifacts. No package
providers or installers run in these smoke pipelines.

A GitHub `release.created` event uploads archives and checksums to that existing
release, using `contents: write` only in the upload job. Pull requests and pushes
that touch release/runtime files build artifacts without publishing. Manual
`workflow_dispatch` defaults to validation only; enabling `publish` requires a
nonempty existing release tag containing this workflow and its scripts.

Publishing starts only after all native build jobs succeed or explicitly skip an
unsupported OpenBSD toolchain. No release tag is created automatically. Linux and
macOS retain their existing release workflows. Native runtime validation happens
in GitHub Actions; adding this workflow does not claim a completed BSD build.

The Wasmtime backport and source revision are documented in
`vendor/wasmtime/FLUENTCI-VENDOR.md` in the source repository.
