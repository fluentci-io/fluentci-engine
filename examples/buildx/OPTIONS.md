# Buildx module arguments

Generated from the committed option snapshot:
`github.com/docker/buildx v0.36.1-desktop.1 83d819cf8237b52ef45a2a9857eeb83a7b10977f`. Includes inherited and experimental options.
Option fields are camelCase versions of Docker's long flags; `type` is named
`type_` in PureScript. Missing flags retain Docker defaults, including default-true
booleans. Explicit `false` is emitted as `--flag=false`. Array values repeat the flag.
Docker byte sizes and durations are strings (for example `2g`, `30s`).

## `buildx`

`Usage:  docker buildx [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.buildx :: BuildxArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `dap`

`Usage:  docker buildx dap [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.dap :: DapArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `debug`

`Usage:  docker buildx debug [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.debug :: DebugArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `invoke` | `--invoke` | `String` |
| `on` | `--on` | `String` |
| `help` | `--help` | `Boolean` |

## `history`

`Usage:  docker buildx history [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.history :: HistoryArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `imagetools`

`Usage:  docker buildx imagetools [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.imagetools :: ImagetoolsArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `policy`

`Usage:  docker buildx policy [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.policy :: PolicyArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `bake`

`Usage:  docker buildx bake [OPTIONS] [TARGET...]`

PureScript: `Buildx.Commands.bake :: BakeArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `targets :: Array String` — Bake targets.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `allow` | `--allow` | `Array String` |
| `call` | `--call` | `String` |
| `check` | `--check` | `Boolean` |
| `file` | `--file` | `Array String` |
| `list` | `--list` | `String` |
| `load` | `--load` | `Boolean` |
| `metadataFile` | `--metadata-file` | `String` |
| `noCache` | `--no-cache` | `Boolean` |
| `policy` | `--policy` | `Array String` |
| `print` | `--print` | `Boolean` |
| `progress` | `--progress` | `String` |
| `provenance` | `--provenance` | `String` |
| `pull` | `--pull` | `Boolean` |
| `push` | `--push` | `Boolean` |
| `sbom` | `--sbom` | `String` |
| `set` | `--set` | `Array String` |
| `var` | `--var` | `Array String` |
| `help` | `--help` | `Boolean` |

## `build`

`Usage:  docker buildx build [OPTIONS] PATH | URL | -`

PureScript: `Buildx.Commands.build :: BuildArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `context :: String` — Build context: path, URL or '-' (stdin is not forwarded by the guest).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `addHost` | `--add-host` | `Array String` |
| `allow` | `--allow` | `Array String` |
| `annotation` | `--annotation` | `Array String` |
| `attest` | `--attest` | `Array String` |
| `buildArg` | `--build-arg` | `Array String` |
| `buildContext` | `--build-context` | `Array String` |
| `cacheFrom` | `--cache-from` | `Array String` |
| `cacheTo` | `--cache-to` | `Array String` |
| `call` | `--call` | `String` |
| `cgroupParent` | `--cgroup-parent` | `String` |
| `check` | `--check` | `Boolean` |
| `file` | `--file` | `String` |
| `iidfile` | `--iidfile` | `String` |
| `label` | `--label` | `Array String` |
| `load` | `--load` | `Boolean` |
| `metadataFile` | `--metadata-file` | `String` |
| `network` | `--network` | `String` |
| `noCache` | `--no-cache` | `Boolean` |
| `noCacheFilter` | `--no-cache-filter` | `Array String` |
| `output` | `--output` | `Array String` |
| `platform` | `--platform` | `Array String` |
| `policy` | `--policy` | `Array String` |
| `progress` | `--progress` | `String` |
| `provenance` | `--provenance` | `String` |
| `pull` | `--pull` | `Boolean` |
| `push` | `--push` | `Boolean` |
| `quiet` | `--quiet` | `Boolean` |
| `resource` | `--resource` | `Array String` |
| `sbom` | `--sbom` | `String` |
| `secret` | `--secret` | `Array String` |
| `shmSize` | `--shm-size` | `String` |
| `ssh` | `--ssh` | `Array String` |
| `tag` | `--tag` | `Array String` |
| `target` | `--target` | `String` |
| `ulimit` | `--ulimit` | `Array String` |
| `help` | `--help` | `Boolean` |

## `create`

`Usage:  docker buildx create [OPTIONS] [CONTEXT|ENDPOINT]`

PureScript: `Buildx.Commands.create :: CreateArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `endpoint :: Maybe String` — Builder context or endpoint.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `append` | `--append` | `Boolean` |
| `bootstrap` | `--bootstrap` | `Boolean` |
| `buildkitdConfig` | `--buildkitd-config` | `String` |
| `buildkitdFlags` | `--buildkitd-flags` | `String` |
| `driver` | `--driver` | `String` |
| `driverOpt` | `--driver-opt` | `Array String` |
| `leave` | `--leave` | `Boolean` |
| `name` | `--name` | `String` |
| `node` | `--node` | `String` |
| `platform` | `--platform` | `Array String` |
| `timeout` | `--timeout` | `String` |
| `use` | `--use` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `dial_stdio`

`Usage:  docker buildx dial-stdio`

PureScript: `Buildx.Commands.dialStdio :: DialStdioArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `platform` | `--platform` | `String` |
| `progress` | `--progress` | `String` |
| `help` | `--help` | `Boolean` |

## `du`

`Usage:  docker buildx du`

PureScript: `Buildx.Commands.du :: DuArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `filter` | `--filter` | `Array String` |
| `format` | `--format` | `String` |
| `timeout` | `--timeout` | `String` |
| `verbose` | `--verbose` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `inspect`

`Usage:  docker buildx inspect [NAME]`

PureScript: `Buildx.Commands.inspect :: InspectArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `name :: Maybe String` — Builder instance name.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `bootstrap` | `--bootstrap` | `Boolean` |
| `timeout` | `--timeout` | `String` |
| `help` | `--help` | `Boolean` |

## `ls`

`Usage:  docker buildx ls`

PureScript: `Buildx.Commands.ls :: LsArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `format` | `--format` | `String` |
| `noTrunc` | `--no-trunc` | `Boolean` |
| `timeout` | `--timeout` | `String` |
| `help` | `--help` | `Boolean` |

## `prune`

`Usage:  docker buildx prune`

PureScript: `Buildx.Commands.prune :: PruneArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `all` | `--all` | `Boolean` |
| `filter` | `--filter` | `Array String` |
| `force` | `--force` | `Boolean` |
| `maxUsedSpace` | `--max-used-space` | `String` |
| `minFreeSpace` | `--min-free-space` | `String` |
| `reservedSpace` | `--reserved-space` | `String` |
| `timeout` | `--timeout` | `String` |
| `verbose` | `--verbose` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `rm`

`Usage:  docker buildx rm [OPTIONS] [NAME...]`

PureScript: `Buildx.Commands.rm :: RmArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `names :: Array String` — Builder instance names.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `allInactive` | `--all-inactive` | `Boolean` |
| `force` | `--force` | `Boolean` |
| `keepDaemon` | `--keep-daemon` | `Boolean` |
| `keepState` | `--keep-state` | `Boolean` |
| `timeout` | `--timeout` | `String` |
| `help` | `--help` | `Boolean` |

## `stop`

`Usage:  docker buildx stop [NAME]`

PureScript: `Buildx.Commands.stop :: StopArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `name :: Maybe String` — Builder instance name.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `use`

`Usage:  docker buildx use [OPTIONS] NAME`

PureScript: `Buildx.Commands.use :: UseArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `name :: String` — Required builder/image name.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `default` | `--default` | `Boolean` |
| `global` | `--global` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `version`

`Usage:  docker buildx version`

PureScript: `Buildx.Commands.version :: VersionArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `dap_build`

`Usage:  docker buildx dap build [OPTIONS] PATH | URL | -`

PureScript: `Buildx.Commands.dapBuild :: DapBuildArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `context :: String` — Build context: path, URL or '-' (stdin is not forwarded by the guest).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `addHost` | `--add-host` | `Array String` |
| `allow` | `--allow` | `Array String` |
| `annotation` | `--annotation` | `Array String` |
| `attest` | `--attest` | `Array String` |
| `buildArg` | `--build-arg` | `Array String` |
| `buildContext` | `--build-context` | `Array String` |
| `cacheFrom` | `--cache-from` | `Array String` |
| `cacheTo` | `--cache-to` | `Array String` |
| `call` | `--call` | `String` |
| `cgroupParent` | `--cgroup-parent` | `String` |
| `check` | `--check` | `Boolean` |
| `file` | `--file` | `String` |
| `iidfile` | `--iidfile` | `String` |
| `label` | `--label` | `Array String` |
| `load` | `--load` | `Boolean` |
| `metadataFile` | `--metadata-file` | `String` |
| `network` | `--network` | `String` |
| `noCache` | `--no-cache` | `Boolean` |
| `noCacheFilter` | `--no-cache-filter` | `Array String` |
| `output` | `--output` | `Array String` |
| `platform` | `--platform` | `Array String` |
| `policy` | `--policy` | `Array String` |
| `progress` | `--progress` | `String` |
| `provenance` | `--provenance` | `String` |
| `pull` | `--pull` | `Boolean` |
| `push` | `--push` | `Boolean` |
| `quiet` | `--quiet` | `Boolean` |
| `resource` | `--resource` | `Array String` |
| `sbom` | `--sbom` | `String` |
| `secret` | `--secret` | `Array String` |
| `shmSize` | `--shm-size` | `String` |
| `ssh` | `--ssh` | `Array String` |
| `tag` | `--tag` | `Array String` |
| `target` | `--target` | `String` |
| `ulimit` | `--ulimit` | `Array String` |
| `help` | `--help` | `Boolean` |

## `debug_build`

`Usage:  docker buildx debug build [OPTIONS] PATH | URL | -`

PureScript: `Buildx.Commands.debugBuild :: DebugBuildArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `context :: String` — Build context: path, URL or '-' (stdin is not forwarded by the guest).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `invoke` | `--invoke` | `String` |
| `on` | `--on` | `String` |
| `addHost` | `--add-host` | `Array String` |
| `allow` | `--allow` | `Array String` |
| `annotation` | `--annotation` | `Array String` |
| `attest` | `--attest` | `Array String` |
| `buildArg` | `--build-arg` | `Array String` |
| `buildContext` | `--build-context` | `Array String` |
| `cacheFrom` | `--cache-from` | `Array String` |
| `cacheTo` | `--cache-to` | `Array String` |
| `call` | `--call` | `String` |
| `cgroupParent` | `--cgroup-parent` | `String` |
| `check` | `--check` | `Boolean` |
| `file` | `--file` | `String` |
| `iidfile` | `--iidfile` | `String` |
| `label` | `--label` | `Array String` |
| `load` | `--load` | `Boolean` |
| `metadataFile` | `--metadata-file` | `String` |
| `network` | `--network` | `String` |
| `noCache` | `--no-cache` | `Boolean` |
| `noCacheFilter` | `--no-cache-filter` | `Array String` |
| `output` | `--output` | `Array String` |
| `platform` | `--platform` | `Array String` |
| `policy` | `--policy` | `Array String` |
| `progress` | `--progress` | `String` |
| `provenance` | `--provenance` | `String` |
| `pull` | `--pull` | `Boolean` |
| `push` | `--push` | `Boolean` |
| `quiet` | `--quiet` | `Boolean` |
| `resource` | `--resource` | `Array String` |
| `sbom` | `--sbom` | `String` |
| `secret` | `--secret` | `Array String` |
| `shmSize` | `--shm-size` | `String` |
| `ssh` | `--ssh` | `Array String` |
| `tag` | `--tag` | `Array String` |
| `target` | `--target` | `String` |
| `ulimit` | `--ulimit` | `Array String` |
| `help` | `--help` | `Boolean` |

## `history_inspect`

`Usage:  docker buildx history inspect [OPTIONS] COMMAND`

PureScript: `Buildx.Commands.historyInspect :: HistoryInspectArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Subcommand positional arguments; prefer the corresponding named export.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `format` | `--format` | `String` |
| `help` | `--help` | `Boolean` |

## `history_export`

`Usage:  docker buildx history export [OPTIONS] [REF...]`

PureScript: `Buildx.Commands.historyExport :: HistoryExportArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `refs :: Array String` — Build record references.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `all` | `--all` | `Boolean` |
| `finalize` | `--finalize` | `Boolean` |
| `output` | `--output` | `String` |
| `help` | `--help` | `Boolean` |

## `history_import`

`Usage:  docker buildx history import [OPTIONS] -`

PureScript: `Buildx.Commands.historyImport :: HistoryImportArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `arguments :: Array String` — Import positional arguments; use the file option instead of interactive stdin.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `file` | `--file` | `Array String` |
| `help` | `--help` | `Boolean` |

## `history_logs`

`Usage:  docker buildx history logs [OPTIONS] [REF]`

PureScript: `Buildx.Commands.historyLogs :: HistoryLogsArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `ref :: Maybe String` — Build record reference.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `progress` | `--progress` | `String` |
| `help` | `--help` | `Boolean` |

## `history_ls`

`Usage:  docker buildx history ls [OPTIONS]`

PureScript: `Buildx.Commands.historyLs :: HistoryLsArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `filter` | `--filter` | `Array String` |
| `format` | `--format` | `String` |
| `local` | `--local` | `Boolean` |
| `noTrunc` | `--no-trunc` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `history_open`

`Usage:  docker buildx history open [OPTIONS] [REF]`

PureScript: `Buildx.Commands.historyOpen :: HistoryOpenArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `ref :: Maybe String` — Build record reference.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `history_rm`

`Usage:  docker buildx history rm [OPTIONS] [REF...]`

PureScript: `Buildx.Commands.historyRm :: HistoryRmArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `refs :: Array String` — Build record references.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `all` | `--all` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `history_trace`

`Usage:  docker buildx history trace [OPTIONS] [REF]`

PureScript: `Buildx.Commands.historyTrace :: HistoryTraceArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `ref :: Maybe String` — Build record reference.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `addr` | `--addr` | `String` |
| `compare` | `--compare` | `String` |
| `help` | `--help` | `Boolean` |

## `imagetools_create`

`Usage:  docker buildx imagetools create [OPTIONS] [SOURCE...]`

PureScript: `Buildx.Commands.imagetoolsCreate :: ImagetoolsCreateArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `sources :: Array String` — Image source references.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `annotation` | `--annotation` | `Array String` |
| `append` | `--append` | `Boolean` |
| `dryRun` | `--dry-run` | `Boolean` |
| `file` | `--file` | `Array String` |
| `metadataFile` | `--metadata-file` | `String` |
| `platform` | `--platform` | `Array String` |
| `preferIndex` | `--prefer-index` | `Boolean` |
| `progress` | `--progress` | `String` |
| `tag` | `--tag` | `Array String` |
| `help` | `--help` | `Boolean` |

## `imagetools_inspect`

`Usage:  docker buildx imagetools inspect [OPTIONS] NAME`

PureScript: `Buildx.Commands.imagetoolsInspect :: ImagetoolsInspectArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `name :: String` — Required builder/image name.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `format` | `--format` | `String` |
| `raw` | `--raw` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `policy_eval`

`Usage:  docker buildx policy eval [OPTIONS] source`

PureScript: `Buildx.Commands.policyEval :: PolicyEvalArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `source :: String` — Required policy evaluation source.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `fields` | `--fields` | `Array String` |
| `file` | `--file` | `String` |
| `platform` | `--platform` | `String` |
| `print` | `--print` | `Boolean` |
| `help` | `--help` | `Boolean` |

## `policy_test`

`Usage:  docker buildx policy test <path>`

PureScript: `Buildx.Commands.policyTest :: PolicyTestArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `path :: String` — Policy test directory.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `filename` | `--filename` | `String` |
| `run` | `--run` | `String` |
| `help` | `--help` | `Boolean` |

## `history_inspect_attachment`

`Usage:  docker buildx history inspect attachment [OPTIONS] [REF [DIGEST]]`

PureScript: `Buildx.Commands.historyInspectAttachment :: HistoryInspectAttachmentArgs -> Either String Plan.Plan`.

Common module parameters: `project` (default `.`), `runDir` (required, fresh),
`docker` (executable, default `docker`), `timeoutMs` (default 900000),
`experimental` (default false, enables `BUILDX_EXPERIMENTAL=1`).

Positional parameter: `references :: Array String` — Optional REF and DIGEST positional arguments.

| Module argument | Docker option | Type |
| --- | --- | --- |
| `builder` | `--builder` | `String` |
| `debug` | `--debug` | `Boolean` |
| `format` | `--format` | `String` |
| `platform` | `--platform` | `String` |
| `type_` | `--type` | `String` |
| `help` | `--help` | `Boolean` |
