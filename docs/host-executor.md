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
- `resources`: names of exclusive resources within this run, such as a writable
  checkout or package-manager prefix. Tasks sharing a resource cannot overlap.
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
separate workspaces. Resource leases currently apply within a run, not across
independent engine processes. Action caching is opt-in through declared inputs/outputs and an explicit
toolchain identity; arbitrary host commands remain uncached by default.

On Unix each task has its own process group, which is killed and reaped on
cancellation or timeout. Background descendants in that group are cleaned up
when the task ends. Processes deliberately creating another session/group are
outside that lifecycle boundary. Non-Unix builds terminate the direct child.
Task stdin is closed; this execution path is intended for noninteractive CI.

The scheduler polls children every 10 ms. This is a correctness baseline, not a
claim of benchmark-leading performance. It bounds process concurrency, not CPU
or memory usage inside each command. Environment-provider preparation, remote
workers, remote artifact storage, and SDK integration
remain follow-up milestones.

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
Corrupt entries trigger a miss; failed commands, missing outputs, and inputs
modified during execution are never published as successful cache entries.

`result.json` includes cache outcome, key, reason, and component hashes. Values
of environment variables are not included in explanations. Cache logs/artifacts
may still contain command output secrets; cache only appropriate tasks. Local
cache storage is trusted, and there is no remote cache protocol yet.

## Live progress and durable events

`--progress auto` selects a unified TUI on a terminal and plain text in CI.
`--progress tui`, `plain`, and `json` select explicitly. The TUI shows concurrent
states, cache hits, elapsed time, and a bounded recent-output pane, refreshed at
most ten times per second. Terminal controls from task output are filtered.

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

The readiness command continues as a health check. Consecutive failures after
readiness fail the service and cancel its active consumers even with
`fail_fast: false`; independent work can continue. An unexpected service exit,
including exit code zero, fails the service. Startup timeout is separate from
optional task `timeout_ms`, which limits its entire lifetime.

Services use `max_services` (default 8), independently of `jobs`. Probes run as
supervised child process groups with timeouts; they do not block scheduler polling.
`ready` and `stopping` states appear in the TUI and JSON event journal, with
corresponding service lifecycle events on the step span. Probe output is discarded;
service stdout/stderr is retained as normal task logs.

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
There is currently no durable waiting queue or submission idempotency key: clients
must not blindly retry submissions after an ambiguous connection failure.

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
