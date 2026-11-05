# Docker Buildx module in PureScript

Buildx runs as native host tasks through the FluentCI PureScript SDK. All runtime
logic is PureScript; JavaScript only bundles and exports the compiled guest.
The example is excluded from Cargo's `examples/*` workspace.

All **33 command exports** have named, statically typed module arguments,
including all **79 distinct long options** from the committed Buildx snapshot,
inherited options and experimental commands. See [OPTIONS.md](OPTIONS.md) for
every function, parameter, type and positional argument.

The compatibility target is the installed CLI captured in
[`buildx-options.json`](buildx-options.json): Buildx **v0.36.1-desktop.1**, upstream
commit `83d819cf8237b52ef45a2a9857eeb83a7b10977f`. Older CLIs may reject newer
options; the module preserves their native diagnostics and exit codes.

## Build the guest

Requires PureScript 0.15.16, Spago 1.0.5, Node.js, Python 3.9+, extism-js
(CI uses 1.7.0), Binaryen, and FluentCI Engine with `execute_plan` (0.6.1+).
The SDK Git revision, package set, Spago lockfile and npm lockfile are committed.
Docker CLI, Buildx and a running Docker daemon are required on the execution host
for image builds; the module uses the existing installation and builder.

```sh
cd examples/buildx
npm ci
npm run build
npm test
```

The build checks generated arguments against the pinned option inventory,
compiles PureScript to JavaScript, bundles CommonJS and compiles
`dist/plugin.wasm`. No Docker daemon is needed to compile or run the pure tests.

## Named module function arguments

`build` accepts a JSON object with **each Buildx option as its own parameter**.
Flag names become camelCase: `--build-arg` → `buildArg`, `--cache-to` → `cacheTo`,
`--no-cache` → `noCache`. Repeatable flags take arrays; boolean flags take actual
booleans. Byte sizes and durations take Docker strings such as `2g` and `30s`.
`provenance` and `sbom` are strings, matching the CLI's attestation shorthand.

From the repository root, using a project containing a Dockerfile:

```sh
mkdir -p runs
fluentci-engine call -m examples/buildx/dist/plugin.wasm build '{
  "project": ".",
  "runDir": "runs/buildx-1",
  "context": ".",
  "file": "Dockerfile",
  "tag": ["my-app:dev"],
  "platform": ["linux/amd64"],
  "buildArg": ["VERSION=dev"],
  "cacheFrom": ["type=local,src=.buildx-cache"],
  "cacheTo": ["type=local,dest=.buildx-cache-next,mode=max"],
  "progress": "plain",
  "pull": false,
  "load": true
}'
```

Choose an existing compatible cache source, or omit `cacheFrom` for a first build.
Use `platform: ["linux/amd64", "linux/arm64"]`, `push: true` and registry tags
for a multi-platform registry build with an appropriately configured builder.
These options follow [Docker's Buildx build API](https://docs.docker.com/reference/cli/docker/buildx/build/).

`bake` exposes all Bake options in the same style:

```sh
fluentci-engine call -m examples/buildx/dist/plugin.wasm bake '{
  "project": ".",
  "runDir": "runs/bake-1",
  "file": ["docker-bake.hcl"],
  "targets": ["app"],
  "set": ["*.platform=linux/arm64"],
  "var": ["VERSION=dev"],
  "print": true
}'
```

Exports also cover `create`, `inspect`, `ls`, `du`, `prune`, `rm`, `stop`, `use`,
`version`, plus `history_*`, `imagetools_*`, `policy_*`, `dap_*` and `debug_*`.
`buildx` and management-group exports accept positional `arguments`; prefer the
specific command export for named options. `help: true` requests command help.
Set `experimental: true` to enable the snapshot's experimental commands/flags.

Common execution parameters are `project` (default `.`), **`runDir`** (required,
fresh), `docker` (executable name/path, default `docker`), and `timeoutMs` (default
900000). Docker credentials, contexts and socket configuration come from the
inherited host environment. Relative `runDir` resolves against the engine
invocation directory; build paths resolve against `project`.

Missing options retain Docker defaults. Explicit false is preserved, including
default-true flags such as `preferIndex`. Unknown fields, wrong types and NUL
arguments fail before execution. Values are literal argv, and positional values
follow `--`. No shell interprets build arguments, paths or Bake overrides.

The guest returns the engine's saved `RunResult` JSON, including stdout/stderr log
paths. Docker failures fail the export while retaining `result.json` and the
native exit code. The executor is noninteractive: guest input is the request
JSON, not Docker stdin. Debugger/DAP/stdio commands, browser UIs and `context: "-"`
are exposed for argument coverage but do not gain an interactive terminal or
stdin bridge; use file-backed inputs and noninteractive Buildx commands here.

## Compose typed PureScript plans

```purescript
import Prelude
import Data.Maybe (Maybe(..))
import Buildx.Commands as Buildx

config = (Buildx.buildArgs "my-project" "runs/build-1")
  { context = "."
  , tag = Just ["my-app:dev"]
  , platform = Just ["linux/arm64"]
  , load = Just true
  }

-- Pure, reusable and composable with FluentCI.Host.Plan helpers:
plan = Buildx.build config
```

Each command has a distinct argument record, a default constructor, a strict JSON
decoder, an argv function and a pure plan builder. `Buildx.execute` is the explicit
effect boundary. This uses the native executor, not the legacy mutable graph or
an embedded Docker/HTTP client.

## Verification and updates

Pure tests check every option against the module schema, repeatable values,
inherited option scope, default/false behavior, literal argv and input rejection.
Engine tests run the compiled Wasm guest against a recording Docker executable,
then optionally build/export a real network-free `FROM scratch` image:

```sh
# From the repository root, with the release engine built:
python3 tests/e2e_buildx.py -v
FLUENTCI_BUILDX_REAL=1 python3 tests/e2e_buildx.py -v
```

To intentionally update the supported Buildx option surface from another installed
CLI (help discovery does not contact the Docker daemon):

```sh
python3 examples/buildx/generate.py --capture
python3 scripts/xtp_codegen.py
```

Review and commit the changed inventory, typed arguments, decoders, Main exports,
option guide and XTP schemas together. Ordinary builds use the snapshot and
never discover flags dynamically. `generate.py --check` and canonical ABI checks
detect drift.
