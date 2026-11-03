# Rust build caches

All Cargo builds remain in release mode. Native Linux/macOS jobs use the shared
`actions/rust-cache` composite action after selecting Rust 1.99.1. It enables:

- sccache 0.16.0 with a local disk backend persisted by actions/cache and an
  absolute compiler wrapper;
- Swatinem/rust-cache for Cargo downloads and compiled dependency artifacts;
- nonincremental compilation, cache persistence after test failures, and cache
  partitions for runner OS/architecture, target, and package-provider backend.

The dependency-cache environment hash excludes the sccache wrapper's absolute
path, which can change between runner jobs; compiler identity and actual compiler
flags remain in the key.

The native macOS release and legacy Linux e2e workflows build directly with Cargo
so these caches are used. Neither workflow deletes Cargo build artifacts before
cache persistence. MacPorts and macports-rs use pinned, SHA-256-verified native binary packages
instead of rebuilding the CLIs; the provider tests still use the compiler caches.

BSD VM jobs restore `.ci-cache/bsd-rust` before boot and copy it back after the
VM run. Cache keys separate guest OS/release/architecture and manifest/source
hashes. The sourced script sets guest-local Cargo home and target paths.
sccache is disabled on FreeBSD, NetBSD, and OpenBSD, including inherited compiler
wrappers, because packaged wrappers can fail to spawn rustc (NetBSD: bad file
descriptor). Cargo download and compiled artifact caching remain enabled, as does
the release toolchain cache. No sccache package is installed in BSD guests. A failed VM run may not reach the action's copyback phase, so only cache
data actually synchronized back can be saved.

Linux release packaging still runs in its existing Dagger container. The release
workflow restores `.ci-cache/linux-release`; the build imports it late, after OS
provisioning layers, then exports the sccache disk cache and Cargo registry/git
downloads after a successful build. Cache mounts are first copied into an
unmounted snapshot directory, because Dagger cannot export mounted caches as
Directory objects. This is separate from native-runner caches.
`FLUENTCI_RUST_CACHE_DIR` enables this transfer; standalone callers can omit it.
Registry mounts now use the official Rust image's `/usr/local/cargo` directory.

The FlakeHub/flakestry jobs publish Nix flakes rather than invoking Cargo directly
and do not use a Rust compiler wrapper. Nix store caching is a separate concern.

The compiler no longer contacts the remote GitHub cache backend for each request.
Archive restore failures are nonfatal, and sccache startup failure clears the
wrapper so compilation can proceed. Cargo dependency caching remains enabled.

Cache hit rates and workflow wall time must be checked in GitHub Actions after
warm runs. Local Rust 1.99.1 release-fixture validation demonstrated a real
sccache hit after cleaning that fixture's Cargo output; it does not measure CI
backend throughput or native BSD/container cache transfers. Cache eviction and
runner image/toolchain updates can still cause cold builds. Compilation of final
binaries and test harnesses can remain non-cacheable even with sccache.

Vendored SuperConsole source and its manifest participate in native sccache and
BSD artifact-cache keys. The native workflow also watches `vendor/superconsole/`
so dependency patches trigger the same OS matrix as engine changes. See
[the vendoring record](../vendor/superconsole/FLUENTCI-VENDOR.md) for the pinned
revision and reproducible patch.

BSD release jobs additionally cache the exact official FreeBSD/NetBSD Rust host
toolchains. They build and smoke-test the full CLI before packaging. The native
OpenBSD compiler comes from its native package repository and uses a compatible
OpenBSD-only runtime dependency selection.
All vendored Rust sources and manifests participate in source cache keys, including
the pinned Wasmtime backport used for NetBSD. See [BSD releases](BSD-RELEASES.md).

MacPorts native-install jobs verify `jq` lookup under the same root/HOME context
as installation before compiling tests. A missing catalog entry after ordinary
HTTPS sync triggers one forced signed snapshot refresh and index regeneration;
a failed recheck stops setup. The selected native/Rust backend is then queried
explicitly, so catalog failures are reported before a Rust test build.

Linux release containers use `rust:1.99.1-trixie` (Debian Trixie). Package index
refresh, installation, and cross-target OpenSSL downloads run in one execution
layer. Cross-target sysroots use Trixie's `libssl3t64` runtime and matching headers.
The release binaries consequently target Trixie's libc baseline.
