#!/bin/sh
set -eu
# Dagger cache mounts cannot be exported as Directory objects. Copy their
# contents into an ordinary, unmounted directory after stopping sccache.
destination=$1
mkdir -p "$destination/sccache" "$destination/registry" "$destination/git"
cp -a "${SCCACHE_DIR:-/root/.cache/sccache}/." "$destination/sccache/"
cp -a "${CARGO_HOME:-/usr/local/cargo}/registry/." "$destination/registry/"
cp -a "${CARGO_HOME:-/usr/local/cargo}/git/." "$destination/git/"
