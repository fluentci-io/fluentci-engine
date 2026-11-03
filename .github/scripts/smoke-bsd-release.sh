#!/bin/sh
set -eu
binary=$1
fixtures=$2
# These fixtures invoke no package managers or installers.
echo 'Smoke: XTP host import roundtrip'
test "$("$binary" call -m "$fixtures/xtp_smoke.wasm" env_roundtrip bsd-release-smoke)" = bsd-release-smoke
echo 'Smoke: XTP JSON export'
"$binary" call -m "$fixtures/xtp_smoke.wasm" echo_json \
    '{"url":"local","function":"echo","args":"BSD release smoke"}' | grep -F '"args":"BSD release smoke"'
for trap in unreachable memory_oob; do
    echo "Smoke: Wasm $trap trap and root-cause diagnostics"
    if "$binary" call -m "$fixtures/$trap.wasm" "$trap" > "$fixtures/$trap.log" 2>&1; then
        echo "Wasm $trap unexpectedly succeeded" >&2
        cat "$fixtures/$trap.log" >&2
        exit 1
    fi
    # Require the actual trap cause, not a loader/missing-export failure.
    case "$trap" in
        unreachable) expected='wasm `unreachable` instruction executed' ;;
        memory_oob) expected='out of bounds memory access' ;;
    esac
    if ! grep -F "$expected" "$fixtures/$trap.log"; then
        echo "Expected Wasm trap cause: $expected; actual error follows:" >&2
        cat "$fixtures/$trap.log" >&2
        exit 1
    fi
done
echo 'Smoke: native host execution and cache restoration'
workspace=$(mktemp -d "$PWD/.bsd-release-smoke.XXXXXX")
(
    cd "$workspace"
    cat > plan.json <<'JSON'
{"version":1,"jobs":2,"inherit_env":false,"cache_dir":"cache","tasks":[{"id":"smoke","program":"/bin/sh","args":["-c","printf bsd-release-smoke > artifact"],"cwd":".","needs":[],"timeout_ms":5000,"cache":{"inputs":[],"outputs":["artifact"],"toolchain":"bsd-release-v1"}}]}
JSON
    "$binary" execute plan.json --run-dir cold --progress plain
    test "$(cat artifact)" = bsd-release-smoke
    rm artifact
    "$binary" execute plan.json --run-dir warm --progress plain
    test "$(cat artifact)" = bsd-release-smoke
    grep -Eq '"outcome"[[:space:]]*:[[:space:]]*"hit"' warm/result.json
)
rm -rf "$workspace"
