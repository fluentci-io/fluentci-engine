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
batch processing. Spans currently cover the host executor; service readiness/shutdown,
metrics, remote context propagation, and server run API integration remain follow-up
work. Queue duration includes scheduler/dependency waiting, with polling granularity.
