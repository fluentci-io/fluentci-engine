# Source from the repository root inside a BSD VM. The directory is synchronized
# back to the runner for actions/cache; never reuse Linux binaries in a guest.
export CARGO_HOME="$PWD/.ci-cache/bsd-rust/cargo"
export CARGO_TARGET_DIR="$PWD/.ci-cache/bsd-rust/target"
export CARGO_INCREMENTAL=0
export SCCACHE_DIR="$PWD/.ci-cache/bsd-rust/sccache"
export SCCACHE_CACHE_SIZE=1G
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR" "$SCCACHE_DIR"
if command -v sccache >/dev/null 2>&1; then
    RUSTC_WRAPPER=$(command -v sccache)
    export RUSTC_WRAPPER
    # Use the disk backend, not Linux-host Actions runtime credentials.
    unset SCCACHE_GHA_ENABLED
else
    echo 'sccache is not available in this BSD package repository; Cargo artifact caching remains enabled.'
fi
