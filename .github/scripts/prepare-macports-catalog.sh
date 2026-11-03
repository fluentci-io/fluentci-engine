#!/bin/sh
set -eu
native_port=${1:-/opt/local/bin/port}
# Run with the same HOME and privileges as the native installation tests.
"$native_port" -N sync
if "$native_port" -q info jq; then
    exit 0
fi
# HTTP snapshot sync uses the source directory mtime to skip a download. A
# fresh directory with a missing/partial index can therefore remain broken.
# Force the signed snapshot refresh and MacPorts' native index regeneration.
echo 'MacPorts cannot resolve jq after sync; forcing a signed catalog refresh.' >&2
"$native_port" -N -f sync
if ! "$native_port" -q info jq; then
    echo 'MacPorts catalog is still unusable; refusing to start install tests.' >&2
    "$native_port" -d info jq || :
    exit 1
fi
