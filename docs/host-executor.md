# Host execution plans

The `execute` command runs an explicit dependency graph on the host. This is the
first execution-core milestone toward a non-containerized Dagger alternative.
The existing GraphQL/Wasm APIs remain available and sequential; they do not yet
compile to this plan format. Their graph execution now validates dependencies
and runs joins only after all predecessors.

```sh
cargo run --release -p fluentci-engine -- execute fixtures/plans/diamond.json --run-dir /tmp/fluentci-example-run
```

The run directory must not already exist, and its parent must exist. Each task
gets separate stdout/stderr files and the run produces `result.json`. Output is
spooled to disk rather than buffered in memory. The CLI exits unsuccessfully if
any task failed, timed out, was skipped, or was cancelled. Ctrl-C cancels running
processes and waits for cleanup before returning.

## Plan version 1

A plan contains `version` (currently 1), `jobs` (positive concurrency bound),
`fail_fast` (default true), `inherit_env` (default true), and `tasks`.

Each task has:

- `id`: unique nonempty ID.
- `needs`: predecessor IDs; duplicates are coalesced.
- `program` and `args`: executable and argument array, passed without shell joining.
- `cwd`: existing working directory. The CLI resolves relative paths against the
  plan file's directory; the Rust library requires absolute paths.
- `env`: task-local environment overrides. No process-wide environment mutation.
- `resources`: names of exclusive resources, such as a writable checkout or
  package-manager prefix. Runs sharing a resource directory coordinate these names.
- `timeout_ms`: optional positive timeout, measured from process launch.

Unknown fields, missing dependencies, cycles, duplicate IDs, and invalid working
directories are rejected before any task runs. Dependencies execute once per run.
Tasks without dependencies are eligible immediately. A join waits for every
predecessor to succeed. Failure skips descendants; with `fail_fast: false`,
independent branches continue. With fail-fast enabled, running siblings are
cancelled and pending independent work is cancelled.

For shell syntax, select it explicitly: `program: "/bin/sh"`,
`args: ["-c", "your script"]`. Otherwise arguments containing spaces, dollar
signs, or shell metacharacters remain literal arguments.

## Per-run copied workspaces

Set `workspace` to copy declared source paths into a private directory for each run:

```json
{
  "version": 1,
  "inherit_env": false,
  "workspace": {"source": "./project", "inputs": ["src", "package.json"]},
  "tasks": [{"id": "build", "program": "/bin/sh", "args": ["-c", "your-build-command"], "cwd": "."}]
}
```

The CLI resolves `workspace.source` against the plan file. Library and HTTP plans
require an absolute source directory. Task `cwd` must be `.` or a relative path
inside that source; its directory must exist in the source at validation time.
Each run copies only selected files/directories into `<run-dir>/workspace`, where
tasks execute and artifacts remain. Nothing is automatically copied back. An
empty input list creates an empty workspace; an unselected task directory is
created empty. Inputs cannot be `.`, contain parent traversal, or overlap.
Symlinks and special files in selected trees are rejected.

Copies preserve file contents and permission modes. Source and copied snapshots
are compared before execution to detect changes during setup. Avoid modifying
source inputs during setup: this is not an atomic filesystem snapshot. Copying
checks cancellation between entries and 64 KiB chunks. Setup finishes before the
task event journal and task spans begin; setup progress events are not yet emitted.

Separate runs have separate copies, while tasks within a run share their workspace
and still need dependencies or resource coordination for shared writes. This is
filesystem layout separation, not an OS sandbox: absolute paths, HOME, network,
and package installations still access host resources. Cache keys retain the
physical working directory by default, so fresh workspaces miss across runs even
with identical inputs unless the task explicitly declares relocatability.

## Functional SDK contract

Constructing a plan is pure data construction. Extending a pipeline should
produce a new task and handle referring to its parent IDs, without executing
commands or changing a shared current pipeline. Multiple branches share parent
IDs; joining branches supplies multiple `needs` entries. Only `execute` performs
I/O. This format is intended as the common target for PureScript, Gleam, Elixir,
and other SDKs; SDK bindings are a subsequent integration milestone.

Results carry explicit `succeeded`, `failed`, `skipped`, `cancelled`, and
`timed_out` states, exit codes where available, error details, durations, and log
paths. Task results retain plan order, regardless of completion order.

## Current boundaries

This executor isolates process environment and working directory configuration,
not filesystem access. Shared writable directories must use resource names or
separate workspaces. Runs using the same resource directory coordinate resource
leases across engine processes. Action caching is opt-in through declared inputs/outputs and an explicit
toolchain identity; arbitrary host commands remain uncached by default.

On Unix each task has its own process group, which is killed and reaped on
cancellation or timeout. Background descendants in that group are cleaned up
when the task ends. Processes deliberately creating another session/group are
outside that lifecycle boundary. Non-Unix builds terminate the direct child.
Task stdin is closed; this execution path is intended for noninteractive CI.

The scheduler waits at most 10 ms between child-process polls. Cache-worker
completion wakes it immediately through a coalescing notification; completions
that arrive before the wait are retained. Results are published before notification
so scheduling does not race worker thread exit. This is a correctness baseline, not a
claim of benchmark-leading performance. It bounds process concurrency, not CPU
or memory usage inside each command. Cache hashing and copying check run
cancellation between 64 KiB chunks, and tree snapshots check between entries.
Cancelled cache work stops the task without falling back to command execution.
Cache preparation, restoration, and saving run on owned background workers.
A task is `running` with cache outcome `preparing` during lookup/restoration and
`storing` during saving. Workers occupy job slots and retain resource leases;
cache misses transfer their leases and action lock directly to the command.
Dependencies are released only after a verified hit or successful cache save.
Journal log tailing waits for preparation to finish, so a rejected restore cannot
advance log cursors before command execution.

For cached tasks, the timeout covers preparation, action-lock retries, command
execution, and saving from the first admitted lookup. Busy action locks release
resource leases and retry after 50 ms; retries do not reset that deadline.
Cancellation, fail-fast, and timeout cancel workers cooperatively. Executor error
cleanup cancels and joins workers before releasing leases. Unrelated task
supervision and service probes continue during cache work. Restoration finishes
or rolls back output replacement once it starts.

This is not a hard real-time deadline: individual filesystem calls can block a
worker and delay its cleanup. Plan validation, process spawning, resource-lock
acquisition, and journal I/O still occur on the scheduler thread.

Native package providers and the PureScript host SDK are available below.
Remote workers, remote artifact storage, and the remaining SDKs are follow-up
milestones.

## Homebrew and MacPorts extensions

`fluentci-ext` provides `homebrew::Homebrew` and `macports::MacPorts` adapters.
They validate existing installations and run commands with a child-local
package-manager environment. They do not install package managers implicitly.
Calling `install(packages)` explicitly installs packages, preserves argument
boundaries, rejects shell fragments/options, and reports nonzero exit status.
Neither adapter invokes sudo implicitly.

Homebrew supports only macOS ARM64 (`/opt/homebrew`) and Linux x86_64
(`/home/linuxbrew/.linuxbrew`), as requested. MacPorts supports macOS only.
The prefixes and environment initialization follow the
[Homebrew documentation](https://docs.brew.sh/Installation) and
[MacPorts guide](https://guide.macports.org/#installing.shell).

MacPorts has two explicitly selected backends:

```rust
use fluentci_ext::macports::{MacPorts, MacPortsBackend};

let native = MacPorts::default(); // /opt/local/bin/port
let rust = MacPorts::new(MacPortsBackend::Rust); // /usr/local/bin/port
let custom = MacPorts::new(MacPortsBackend::Rust)
    .with_executable("/absolute/custom/install/port");
```

The Rust option targets [macports-rs](https://github.com/tsirysndr/macports-rs),
whose installed CLI is also named `port`. The selected executable is pinned;
there is no silent backend fallback based on PATH. A custom executable supports
`PORT_RS_INSTALL_DIR` deployments. Prefix permissions and provisioning remain
explicit host configuration. These adapters are Rust extension APIs; fluent
GraphQL/Wasm and functional-language provider bindings remain to be integrated.

## Validation

```sh
cargo test --release -p fluentci-executor
cargo test --release -p fluentci-core --lib
cargo test --release -p fluentci-ext --lib homebrew
cargo test --release -p fluentci-ext --lib macports
cargo check --release -p fluentci-engine
```

Tests cover dependency diamonds, shared ancestors, missing/cyclic dependencies,
bounded concurrency, exclusive resources, concurrent run environments, literal
arguments, both output streams, fail-fast/continue behavior, timeouts,
cancellation of descendants, empty plans, spawn failures, and backend selection.
No package installation is performed by the adapter tests.


## Local action caching

Set a plan-level `cache_dir` (relative to the plan file in the CLI),
`inherit_env: false`, and an opt-in task policy:

```json
"cache": {
  "inputs": ["src", "package-lock.json"],
  "outputs": ["dist"],
  "toolchain": "node-22.13.1-with-locked-build-dependencies"
}
```

Paths must be nonoverlapping, relative to the task working directory, and cannot
contain `.` or `..`. Files and directory trees, including empty directories and
Unix executable permissions, are supported. Symlinks and special files are
currently rejected rather than followed. Declared outputs are cleaned before a
cache miss runs: outputs must not double as undeclared inputs.

Keys include input contents and permissions, executable contents, arguments,
explicit environment, caller-supplied toolchain identity, OS/architecture,
working directory, output declarations, and dependency keys. A dependency without
a cache policy forces a new descendant action key for that run. Files used by
interpreters, compilers, or external services must be represented by declared
inputs/toolchain identity; this is an explicit cacheability contract, not a
hermetic sandbox. Side-effecting deployment and publishing tasks should not opt in.

Blobs are content-addressed with SHA-256. Writes are staged and atomically
published. Action locks deduplicate concurrent identical executions on Unix.
Restoration verifies all staged artifacts and logs before replacing outputs.
Output replacement keeps backups and rolls back earlier replacements if a later
filesystem operation fails. If rollback itself fails, backups are retained and
the cache diagnostic reports their location. This is not crash-atomic across
multiple output paths, and concurrent workspace mutation is not supported. A
rejected restore falls back to normal execution, which cleans declared outputs.
Corrupt entries trigger a miss; failed commands, missing outputs, and inputs
modified during execution are never published as successful cache entries.

Within a plan, overlapping declared cache paths require a direct or transitive
`needs` relationship whenever either task writes the path. This includes nested
paths and aliases of task working directories. Shared inputs are allowed. A
single job slot or resource lock does not define dataflow order, so neither
bypasses this check. Validation rejects conflicts before creating the run or
starting commands. Undeclared accesses and conflicts between separate runs still
require caller-managed workspace separation or shared resource leases; this
validation does not provide filesystem isolation.

`result.json` includes cache outcome, key, reason, and component hashes. Values
of environment variables are not included in explanations. Cache logs/artifacts
may still contain command output secrets; cache only appropriate tasks. Local
cache storage is trusted, and there is no remote cache protocol yet.

## Live progress and durable events

`--progress auto` selects a unified TUI on a terminal and plain text in CI.
`--progress tui`, `plain`, and `json` select explicitly. The TUI shows concurrent
states, cache hits, elapsed time, and a bounded recent-output pane, refreshed at
most ten times per second. Failed and timed-out tasks take priority over completed
work when the terminal cannot fit every row; running, ready and stopping services
also stay ahead of completed work. Failure reasons remain in their task rows even
after later logs displace them from the recent-output pane. Row labels/details and
tail entries are kept to one line, and terminal controls from task output are filtered.
Long rows are clipped to the terminal width; full reasons remain in the result and journal.

Every run records ordered `events.jsonl` events: run start, task states, log
chunks, and run finish. The CLI tails this journal independently of execution;
slow display output does not block subprocess execution. JSON progress produces
only event records on stdout, suitable for SDKs/server transport. Original
stdout/stderr files retain exact bytes; streamed text is a lossy UTF-8 display
view. Event storage currently duplicates log data and has no retention policy.

## BSD package managers and native CI

`fluentci_ext::bsd::BsdPackages` exposes `freebsd()`, `netbsd()`, and `openbsd()`:

| OS | Executable | Noninteractive install |
| --- | --- | --- |
| FreeBSD | `/usr/local/sbin/pkg` | `pkg install -y` |
| NetBSD | `/usr/pkg/bin/pkgin` | `pkgin -y install` |
| OpenBSD | `/usr/sbin/pkg_add` | `pkg_add -I` |

Adapters reject the wrong OS, preserve package argument boundaries, use the
base `/bin/sh`, and prepend the appropriate package prefix in the child only.
Package-manager bootstrap, repository configuration, and privilege elevation
are explicit provisioning steps. Command semantics follow the
[FreeBSD handbook](https://docs.freebsd.org/en/books/handbook/ports/),
[pkgin project](https://pkgin.net/), and
[OpenBSD manual](https://man.openbsd.org/pkg_add).

The `Host executor` workflow runs Linux/macOS tests directly on hosted runners.
BSD jobs boot real guest kernels using pinned VM Actions on Ubuntu, install Rust,
then execute executor/cache tests and opt-in package installation tests. BSD
jobs intentionally avoid the legacy CLI's Wasmtime stack. These workflows are
configured but their remote results must be checked before claiming native BSD
support is verified. Native MacPorts/macports-rs installation jobs are still to
be added; their adapter fixtures are already covered on macOS.

Run local end-to-end tests with `cargo build --release -p fluentci-engine` followed by
`python3 tests/e2e_host.py -v`. Native installation tests are `#[ignore]` by
default and must only be enabled on disposable provisioned hosts.

### OpenTelemetry per step

`execute` uses the existing OTLP exporter configuration (for example,
`OTEL_EXPORTER_OTLP_ENDPOINT` and `OTEL_EXPORTER_OTLP_PROTOCOL`). Each invocation
creates a `pipeline.run` span and a `pipeline.step` span for every task, including
cached, skipped, and cancelled tasks. Step spans include queue duration, execution
duration, cache outcome, exit code, and terminal status. Dependency events identify
predecessor spans. Execution JSONL events carry matching trace/span IDs when a
provider is enabled, including log events. Raw log text, command arguments, working
directories, and environment values are not exported as span attributes.

The library uses the host's global OpenTelemetry provider and current parent context;
it does not initialize or shut down an exporter. The CLI flushes its exporter on a
blocking worker before exiting, including on failed runs. Exporters should use bounded
batch processing. Spans currently cover the host executor; metrics, remote context propagation, and server run API integration remain follow-up
work. Queue duration includes scheduler/dependency waiting, with polling granularity.


### Long-running services

A task with a `service` object remains alive while its direct and transitive
consumers run. Dependencies on a service are released when its readiness command
succeeds, rather than when the service exits. For example, a PostgreSQL task can
run `postgres` with a readiness command of `pg_isready` against an explicitly
configured port. All service/probe commands use argv arrays, the task working
directory, and its child-local environment.

```json
"service": {
  "readiness": {
    "program": "/path/to/pg_isready",
    "args": ["-h", "127.0.0.1", "-p", "55432"],
    "interval_ms": 250,
    "timeout_ms": 1000
  },
  "startup_timeout_ms": 30000,
  "shutdown_timeout_ms": 5000,
  "failure_threshold": 3
}
```

By default, the readiness command continues as a health check. Set optional
`service.health` to a separate probe with its own `program`, `args`, `interval_ms`,
and `timeout_ms` to replace it after readiness succeeds. Health checks never gate
initial readiness: the first health check is scheduled after its own interval
following successful readiness. Both probe types have the same environment and
working directory as the service and use supervised process groups. Consecutive failures after
readiness fail the service and cancel its active consumers even with
`fail_fast: false`; independent work can continue. An unexpected service exit,
including exit code zero, fails the service. Startup timeout is separate from
optional task `timeout_ms`, which limits its entire lifetime.

Services use `max_services` (default 8), independently of `jobs`. Probes run as
supervised child process groups with timeouts; they do not block scheduler polling.
`ready` and `stopping` states appear in the TUI and JSON event journal, with
corresponding service lifecycle events on the step span. Probe output is retained
in separate diagnostic streams; service stdout/stderr is retained as normal task logs.
Health-threshold errors include the consecutive-failure count and last probe exit
status or timeout. Spawn and log-open failures identify readiness versus health
and retain the OS error. Startup timeouts include the last failed readiness attempt
when available. These lifecycle reasons appear in task results and journal state
events without echoing probe arguments or environment values. Raw probe output may
still contain whatever the probe itself prints.

When consumers finish, or execution is cancelled, dependent services are stopped
before their providers. Shutdown signals the Unix process group with TERM, waits
up to `shutdown_timeout_ms`, then uses KILL and reaps the child. Probe process groups
are also cleaned up. Services and their consumers cannot be cached because mutable
service state is outside the action-cache contract. Sharing an exclusive resource
between a service and its consumer is rejected. Capacity/resource configurations
that leave only ready services holding leases needed by pending work fail with a
cleanup diagnostic instead of hanging.

This implementation is Unix-first. Ports, data directories, and endpoint environment
variables must currently be chosen explicitly by the caller; cross-run endpoint
allocation, automatic restart policies, and server-managed persistent services are
not yet implemented. Service programs must stay in the foreground; deliberately
escaping process groups is outside the supervision guarantee.

### Independent server runs

The new run API uses the same executor, cache, services, and JSON plan as local
`execute`. Start it with a private data directory and an authentication token:

```sh
export FLUENTCI_RUN_TOKEN='replace-with-a-long-random-token'
cargo run --release -p fluentci-engine -- serve-runs \
  --listen 127.0.0.1:6881 --data-dir /path/to/run-store --max-runs 4
```

Every request requires `Authorization: Bearer <token>`. Server plans must set
`inherit_env: false` and supply absolute working/cache directories on the server.
The server token and other ambient credentials are therefore not inherited by
submitted task processes. This is a trusted host execution API: an authorized
client can run host commands. It does not provide tenant isolation or a sandbox.
Use a TLS/authenticated gateway when accessing it over an untrusted network.

| Request | Behavior |
| --- | --- |
| `POST /v1/runs` with a plan | Validates and persists the plan; returns 202, a run ID, and `Location` |
| `GET /v1/runs/{id}` | Reads persisted running/succeeded/failed/cancelled/interrupted state |
| `GET /v1/runs/{id}/result` | Reads the executor result; 404 until it exists |
| `POST /v1/runs/{id}/cancel` | Requests cancellation and service teardown; repeat requests are safe |
| `GET /v1/runs/{id}/events?cursor=0` | Reads up to 4 MiB of complete JSONL events |
| `GET /v1/runs/{id}/logs/{index}/stdout?cursor=0` | Reads up to 1 MiB of exact stdout bytes; `stderr` also supported |

`index` is the task's zero-based position in the submitted plan. Journal/log reads
return `X-Next-Cursor`; pass it on the next poll to resume without replaying old
bytes. Event cursors must be at newline boundaries. Poll run state to determine
completion, then drain remaining events/logs. Responses are marked `no-store`.
The original binary logs remain on disk; JSON event text remains a lossy UTF-8
view. Events include trace/span IDs when OpenTelemetry is enabled. Incoming W3C
`traceparent`/`tracestate` headers become the parent context for run and step spans.

Admission is bounded by `--max-runs`; excess submissions return 429 with
`Retry-After: 1`. Each plan is limited to 64 ordinary jobs, 32 service slots,
10,000 tasks, and a 1 MiB JSON body. Plan validation and durable admission run on
blocking workers, leaving HTTP workers available for status and log requests.
There is no durable waiting queue. `POST /v1/runs` accepts an optional
`Idempotency-Key` header (1–128 ASCII letters, digits, hyphens, or underscores).
The server records the key before executing commands. Retrying with the same key
and plan (after applying field defaults; object-key order is ignored, array order
is significant) returns `202`, the original `Location`, and its current state,
even when capacity is full or the original workspace no longer exists. Reusing a
key for another plan returns `409`. Accepted keyed responses echo `Idempotency-Key`
so clients can detect older servers that ignore the header.

Keys are case-sensitive, scoped to the data directory, survive restart, and have
no automatic expiry. Their storage filenames use SHA-256, including on
case-insensitive filesystems. Use a unique key per intended execution and retain both that key and the
immutable plan for retries. Interrupted or failed runs are returned as-is; a retry
does not restart them. This prevents duplicate admission, not duplicate effects
inside task commands or guaranteed completion after a crash. Capacity/validation
rejections do not reserve keys. Without a key, clients must not blindly retry an
ambiguous submission. Do not remove submission records while callers may retry.

Plans and state files are atomically replaced in private run directories. A Unix
file lock prevents two servers from opening the same store. Graceful server shutdown
cancels active executions, waits for process/service cleanup, persists results, and
flushes telemetry. After restart, finished reports are retained and incomplete
metadata is reconciled to a terminal report or `interrupted`. Hard kills or machine
crashes cannot guarantee child cleanup on every supported OS; interrupted runs are
not automatically rerun. Run-store garbage collection is not yet implemented.

`serve-runs` is separate from the existing GraphQL `serve` command. The legacy SDK
server retains its existing semantics until the SDK compatibility path is migrated;
the new run API does not share its mutable current-pipeline graph.


### Cross-run resource leases

On Unix, `resources` now use process-shared advisory file locks. The default
coordination directory is `$HOME/.cache/fluentci/resources`; set the plan's
`resource_dir` to an absolute private directory to use another namespace. Local
CLI plans may use a relative `resource_dir`, resolved beside the plan file.
CLI and server runs with the same directory coordinate with each other.

A task acquires its complete sorted, deduplicated resource set before cache lookup,
restoration, or execution. If any lease is busy, partial acquisition is released
and retried without occupying an execution slot. Leases cover cache publication
and child/service cleanup, including error unwinding. Different resource names
can execute concurrently. This is coordination for cooperating processes, not
filesystem isolation: all writers must declare the same names and namespace.

Tasks may set `resource_wait_timeout_ms` to bound contention separately from
execution `timeout_ms`. The default is no wait deadline. Cancellation remains
available while waiting. JSON events/results expose `wait_reason`; plain/TUI
output displays contention, and step spans record resource waiting. Retry cadence
is 50 ms; ordering is not FIFO and starvation freedom is not guaranteed. Use a
wait deadline when service dependencies across runs could form a cycle.

The directory must be private (0700) and owned by the executing user. Resource
names are hashed into stable file names; lock files must not be deleted while any
run uses the namespace, because replacing an inode would create a separate lock.
Do not put the namespace inside directories that tasks delete or overwrite.
Cached tasks reject lock storage that overlaps declared inputs/outputs.

These locks do not guarantee child termination after a hard engine kill or machine
failure. The OS releases the engine's locks on death, but orphaned child commands
may remain on some hosts. Use graceful cancellation/shutdown until independent
crash supervision is implemented; do not treat locks as a sandbox boundary.


### PureScript SDK

The sibling `purescript-fluentci` repository now has `FluentCI.Host.Plan` pure
receiver-last builders, `FluentCI.Host.Client` for the run API, and
`FluentCI.Host.Local` for direct Node execution. Plans compose with `#`; execution
returns `Aff (Either RunError ...)`. Services, action caches, resource leases,
raw logs, event cursors, cancellation, and resumed run handles use the same wire
contract. Existing `FluentCI.Pipeline`/GraphQL APIs remain available separately.

After building this engine with `cargo build --release -p fluentci-engine`, run
`python3 test/e2e_host.py` in the sibling SDK repository. Set `FLUENTCI_ENGINE_BIN`
to this release binary when the repositories are not siblings. This integration
has been exercised locally; cross-repository CI and the other language adapters
remain follow-through work.

### Package installation and toolchain environments in plans

Package installation is an explicit DAG task. For example:

```json
{
  "id": "packages",
  "cwd": "/workspace/app",
  "package_install": {
    "provider": { "manager": "homebrew" },
    "packages": ["jq"]
  }
}
```

A consumer declares both the dependency and its environment:

```json
{
  "id": "check",
  "needs": ["packages"],
  "cwd": "/workspace/app",
  "program": "jq",
  "args": ["--version"],
  "package_environment": [{ "manager": "homebrew" }]
}
```

Supported `manager` values are `homebrew`, `macports`, `freebsd`, `netbsd`, and
`openbsd`. Homebrew is restricted to macOS ARM64 and Linux x86_64. MacPorts is
macOS-only; select `"backend": "rust"` for macports-rs or `"native"` (the default).
BSD providers require the matching BSD kernel. An optional absolute `executable`
selects an explicitly provisioned manager binary, including custom macports-rs
install locations; it does not change the conventional package installation prefix.

The executor validates providers before executing commands and never bootstraps a
manager or invokes sudo automatically. Install tasks use literal argv and appear
as normal steps in logs, results, the TUI, and OpenTelemetry. Set `timeout_ms` and
needed environment values such as `HOME` explicitly for your CI host.

Provider environments prepend conventional package `bin`/`sbin` directories, then
the manager executable's directory, to task-local PATH. Package binaries take
precedence over unrelated tools next to a custom manager executable. Existing
explicit task PATH is preserved as the suffix; an explicit-environment plan otherwise gets the system
path `/usr/bin:/bin:/usr/sbin:/sbin`. Multiple providers use array order as PATH
precedence. Homebrew also receives `HOMEBREW_PREFIX` and `NONINTERACTIVE=1`.
Direct manager commands (`program: "port"`, for example) are pinned to the chosen
binary. Arbitrary shell scripts are not rewritten; use absolute paths when calling
custom-named manager executables or selecting the Rust `port` backend from a
shell. Keg-only formula paths and custom installation prefixes must be configured separately; this is not a general
replacement for every manager-specific shell initialization hook.

Installers automatically take an exclusive `package-prefix:<canonical-prefix>`
lease. Consumers take a shared lease for that prefix, so they can run in parallel
while coordinated updates wait. `shared_resources` also exposes shared leases for
other explicitly declared read-only resources; an exclusive request for the same
name wins when both are present. All cooperating runs must share `resource_dir`.
Commands using `package_environment` must not mutate package prefixes; use an
installation task or an explicit exclusive prefix lease for updates. Updates
outside the engine's lease protocol are not coordinated.

Install tasks are never action-cached and cannot also be command/service tasks.
A consumer depending on an uncached installation receives a run-specific dependency
key, so it does not claim a cross-run cache hit based on mutable package state.
For cached consumers, provision a pinned toolchain separately, declare its identity
in `cache.toolchain`, and use `package_environment` without an installation step in
the cached dependency graph. Complete automatic package-state snapshots and verified
toolchain identities remain follow-through work.

Fixture tests exercise command selection, argv, failures, parallel readers, exclusive
updates, child-local environments, and cached consumers. The native CI matrix now
includes an opt-in `native_package_plan_install` test that installs `jq` and runs it
through the provider environment on supported Homebrew and BSD hosts. These jobs
are configured but have not yet been executed remotely. MacPorts provisioning and
native macports-rs CI coverage remain pending.


### Native MacPorts CI

The host-executor workflow has separate macOS ARM64 and x86_64 jobs for native
MacPorts and the optional Rust backend. It builds [MacPorts base v2.12.7](https://github.com/macports/macports-base/tree/dcf596a2d661edf370cd448d65db8fe99f8524e9)
using the [official source installation procedure](https://guide.macports.org/#installing.macports.source)
and builds [macports-rs at a pinned commit](https://github.com/tsirysndr/macports-rs/tree/5942733500137883ba642db6e8261f57907c531e)
with `cargo build --release --locked`. Both configurations use base-provided
configuration and a synced ports tree; this does not test standalone Rust-backend
bootstrap. The ports tree and package archives remain upstream network inputs.

Compilation runs without elevation. Only provisioning and the already-built
ignored native tests run as root on disposable GitHub runners. Each job starts
without `/opt/local/bin/jq`, installs it through the chosen provider, verifies that
PATH selects that exact binary, then exercises the legacy extension adapter.
These jobs are configured; successful remote execution is still required before
claiming native backend coverage. Do not run these installation steps on a normal
development host.

## Relocatable action caches

Set `cache.relocatable: true` only when the command's outputs **and stdout/stderr**
are independent of its physical working directory. It defaults to false. This
explicit contract enables cache hits and action-lock deduplication across copied
run workspaces and across distinct checkouts:

```json
"cache": {
  "inputs": ["input"],
  "outputs": ["output"],
  "toolchain": "my-complete-toolchain-v1",
  "relocatable": true
}
```

Only the working-directory component of the key changes: it becomes a fixed
relocatable identity. Inputs, output paths, executable contents/mode, exact argv,
environment, platform, toolchain, and dependency keys still participate. Default
path-sensitive keys retain their existing identity and cannot collide with this
mode. Absolute paths in arguments/environment are not rewritten; they may still
prevent reuse. A path-sensitive predecessor's key can also prevent cross-run hits.

The engine cannot infer or verify relocatability for arbitrary host commands.
Commands using `pwd`, embedding source paths in binaries/debug information, reading
undeclared sibling files, or logging absolute paths do not meet this contract
without appropriate toolchain configuration. Restored logs are the original bytes,
not rewritten for the new run. Cacheable commands must not require side effects
outside declared outputs. Keep the default for path-dependent commands; declaring
relocatability incorrectly can return incorrect artifacts. This is an explicit
extension to the existing declared-input/toolchain contract, not hermetic execution.

## Compare cache decisions

Compare two saved run directories or `result.json` files without executing a plan:

```sh
fluentci-engine cache-diff ./previous-run ./current-run
fluentci-engine cache-diff ./previous-run/result.json ./current-run/result.json --json
```

Tasks are matched by ID, independent of report order. Each task is `changed`,
`unchanged`, `added`, `removed`, or `unavailable` (one or both runs did not produce
an action key, for example an uncached, skipped, or early-failed task). Changed
component names identify differences in inputs, command, environment, toolchain,
outputs, dependencies, working directory, executable, or platform. Values and raw
error messages are not printed. Plain output quotes task/component names so control
characters cannot become terminal escapes; JSON output has a versioned envelope.
Malformed reports and duplicate task IDs fail instead of silently dropping tasks.

This compares recorded identities, not the live filesystem or cache store. An
unchanged key does not guarantee a hit: entries may be absent, evicted, or rejected
for integrity errors. Inspect the original result's cache outcome/reason for that
case. A changed dependency digest identifies upstream invalidation but does not
recursively explain that dependency; compare its task entry as well. Only digests
are stored, so this command cannot name individual changed input files or reveal
which environment variable changed.

## Service probe diagnostics

Service results include `probe_logs` with `stdout` and `stderr` paths under the
run directory (`<index>.probe_stdout` and `<index>.probe_stderr`). Both readiness
and recurring health attempts append their raw output, preceded by a short
`--- readiness probe ---` or `--- health probe ---` marker. Non-service results
omit this field. Files are created when the first probe is attempted; services
cancelled or skipped before probing may have no probe log files.

The authenticated raw log endpoint accepts `probe_stdout` and `probe_stderr` in
addition to `stdout` and `stderr`. It uses the same byte cursor and 1 MiB response
limit; append-only files allow resuming while checks continue. These files are
separate from the daemon's output. Probe processes write directly to disk, avoiding
scheduler pipe-draining work and unbounded in-memory output buffers. As with task
logs, total disk usage is not capped yet. Probe output is also streamed through the execution journal and plain/TUI progress
views, labeled `probe_stdout` and `probe_stderr`. Each stream has its own cursor and
the same 64 KiB per scheduler-tick limit as command output; the final drain records
remaining bytes before the run-finished event. Journal text is lossy UTF-8 for
display; use the files or raw HTTP streams for binary fidelity. Probe journal
events carry the service step trace/span IDs, but raw output is not added to span
attributes.

## Run-store shutdown admission

Shutting down the run store permanently closes admission for that instance before
cancelling and joining its registered workers. Admission rechecks closure after
plan validation while holding the same lock used to register workers, so a request
that was validating during shutdown cannot start work afterward. Such submissions
receive HTTP 503 (`run server is shutting down`). Concurrent shutdown callers wait
for the same cleanup to finish. Restart with a new store/server instance to admit
new runs; this does not introduce a durable admission queue or crash supervision.

## Result publication and restart recovery

The executor writes `result.json` through a temporary file in the run directory,
flushes and syncs it, then atomically renames it and syncs the directory on Unix.
Readers therefore see a complete result or no published result. This protects
report publication; it does not make arbitrary task outputs crash-durable.

On restart, a run still marked running is reconciled only from a report with the
same ordered task IDs/count as its saved plan and terminal statuses for every task.
A missing, truncated, malformed, mismatched, or nonterminal report marks that run
interrupted and does not prevent other runs recovering or new work being admitted.
Reports are retained for diagnosis, and recovery never reruns commands. New reports
persist `outcome` as `succeeded`, `failed`, or `cancelled`. The executor captures it
once before publication: all successful tasks mean succeeded; otherwise an observed
external cancellation means cancelled; other unsuccessful runs mean failed. A
fail-fast run remains failed even when it cancels sibling tasks. The live server
and restart recovery use the same published outcome. Late cancellation requests do
not rewrite it. Recovery rejects unknown outcomes or outcomes inconsistent with
whether all tasks succeeded.

Older reports without `outcome` remain readable: recovery reports succeeded when
all tasks succeeded and failed otherwise, since those reports cannot distinguish
external cancellation. These recovery tests seed persisted crash-window states;
they do not establish independent orphan-process cleanup after an actual hard
server crash.

## Discover previous runs

`GET /v1/runs?limit=50` returns authenticated history as
`{"runs": [...], "next_cursor": null | "..."}`. Follow `next_cursor` with
`GET /v1/runs?limit=50&before=<URL-encoded cursor>`. Limits are 1–200. Entries use
creation time descending, with descending run ID as a deterministic timestamp tie
breaker. Each state now persists `created_at_ms` (Unix milliseconds); it is retained
through completion, replay, and restart. Legacy states use zero for unknown time
and sort after timestamped states. This is a live listing, not a snapshot: statuses
can change between pages and newly created entries are discovered by refreshing
the first page. Ordering follows the host clock; clock adjustments can affect where
new entries appear.

History uses an in-memory ordered index of creation timestamps and run IDs, rebuilt
from readable persisted states when the store opens and updated when admission
persists a new state. It uses O(number of runs) index memory. A page selects at most
`limit + 1` keys under a read lock, releases the lock, and reads at most `limit` state
files on a blocking worker. It does not scan the store or read plans/logs per page;
statuses are read fresh instead of cached. Key selection costs O(log N + limit).
Startup still scans the store; an on-disk index and retention remain future work.

The store owns membership and creation timestamps. Do not edit, import, or remove
run directories while it is open: external additions appear only after reopening,
and selected missing/malformed/mismatched states produce an error rather than a
silently incomplete page. UUID directories are inspected without following directory
symlinks during discovery. History uses the same bearer authentication as execution.
