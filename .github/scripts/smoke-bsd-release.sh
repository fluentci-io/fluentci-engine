#!/bin/sh
set -eu
binary=$1
fixtures=$2
# These fixtures invoke no package managers or installers.
test "$("$binary" call -m "$fixtures/xtp_smoke.wasm" env_roundtrip bsd-release-smoke)" = bsd-release-smoke
"$binary" call -m "$fixtures/xtp_smoke.wasm" echo_json \
    '{"url":"local","function":"echo","args":"BSD release smoke"}' | grep -F '"args":"BSD release smoke"'
for trap in unreachable memory_oob; do
    if "$binary" call -m "$fixtures/$trap.wasm" "$trap" > "$fixtures/$trap.log" 2>&1; then
        echo "Wasm $trap unexpectedly succeeded" >&2
        exit 1
    fi
    # Require an actual trap, rather than accepting a loader/missing-export failure.
    case "$trap" in
        unreachable) grep -i 'unreachable' "$fixtures/$trap.log" ;;
        memory_oob) grep -i 'out of bounds' "$fixtures/$trap.log" ;;
    esac
done
workspace=$(mktemp -d "$PWD/.bsd-release-smoke.XXXXXX")
(
    cd "$workspace"
    cat > plan.json <<'JSON'
{"version":1,"jobs":2,"cache_dir":"cache","tasks":[{"id":"smoke","program":"/bin/sh","args":["-c","printf bsd-release-smoke > artifact"],"cwd":".","needs":[],"timeout_ms":5000,"cache":{"inputs":[],"outputs":["artifact"],"toolchain":"bsd-release-v1"}}]}
JSON
    "$binary" execute plan.json --run-dir cold --progress plain
    test "$(cat artifact)" = bsd-release-smoke
    rm artifact
    "$binary" execute plan.json --run-dir warm --progress plain
    test "$(cat artifact)" = bsd-release-smoke
    grep -Eq '"outcome"[[:space:]]*:[[:space:]]*"hit"' warm/result.json
)
rm -rf "$workspace"
