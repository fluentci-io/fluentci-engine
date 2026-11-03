# Gleam Wasm plugins

Use `fluentci/plugin` to call the PDK functions exposed by FluentCI from inside a Wasm guest.

```gleam
plugin.pipeline("greet") |> plugin.exec("/usr/bin/printf", ["%s", input]) |> plugin.stdout
```

Install Gleam 1.19.2, Node.js, and [extism-js v1.7.0](https://github.com/extism/js-pdk).

```sh
cd sdk/gleam/demo-plugin
npm ci
npm run build
fluentci-engine call -m dist/plugin.wasm greet 'hello from gleam'
```

The build compiles to JavaScript, bundles ES2020 CommonJS, and compiles an Extism
Wasm guest. It calls `Host`/`Memory` guest globals, without embedding a host runtime
or using an HTTP client. Dependencies must fit the Extism JavaScript sandbox;
Node/Deno OS APIs are unavailable inside Wasm. `imports.d.ts` comes from the
canonical `schema/host.json`.

## Existing APIs

# FluentCI Gleam SDK

[![Package Version](https://img.shields.io/hexpm/v/fluentci)](https://hex.pm/packages/fluentci)
[![Hex Docs](https://img.shields.io/badge/hex-docs-ffaff3)](https://hexdocs.pm/fluentci/)

The FluentCI Gleam SDK provides a set of APIs to interact with FluentCI in Gleam, it allows you to write CI/CD pipelines in Gleam.

## 🚚 Installation

```sh
gleam add fluentci
```

## Pure host plans (JavaScript target)

`fluentci/host` builds immutable data for the bounded host executor. Functions take
the pipeline value first, so they compose with Gleam's `|>`:

```gleam
import fluentci/host
import gleam/io

pub fn main() {
  let build = host.task("build", "/bin/sh", ".")
    |> host.with_args(["-c", "printf artifact > output"])
    |> host.with_relocatable_cache(host.cache([], ["output"], "my-toolchain-v1"))
  let check = host.task("check", "/bin/sh", ".")
    |> host.with_args(["-c", "test -s output"])
    |> host.needs([build])
  let assert Ok(json) = host.plan([build, check])
    |> host.with_jobs(2)
    |> host.with_workspace("/absolute/project", [])
    |> host.with_cache_directory("/absolute/cache")
    |> host.compile
  io.println(json)
}
```

Save the generated JSON and execute it with the release engine:

```sh
fluentci-engine execute plan.json --run-dir /tmp/new-run --progress auto
```

Builders cover explicit argv/environment, dependencies, concurrency, resource
leases and wait timeouts, cache policies, copied workspaces, service readiness and
health, and Homebrew/MacPorts/BSD provider declarations. `install` creates an
explicit package installation task; `with_package_environment` selects environments
for consumers, and `needs` supplies the ordering. `provider(MacPortsRust)` selects
the optional Rust backend. Platform support and native prerequisites are enforced
by the engine; the SDK does not install package managers automatically.

Construction and `compile` perform no I/O. Compilation checks IDs, missing
references, commands, and basic limits; the engine still validates cycles, paths,
cache contracts, platform support, and service/resource constraints. An explicit
`with_relocatable_cache` asserts path-independent outputs and logs; keep
`with_cache` for path-dependent commands. Workspaces separate run files, not host
permissions. Unused optional fields are omitted for older-engine compatibility.

### Execute locally with typed results

`fluentci/host/local` runs the engine directly on Node or Deno (with filesystem,
environment and subprocess permissions). No additional dependencies are required:

```gleam
import fluentci/host
import fluentci/host/local
import gleam/javascript/promise

pub fn main() {
  let task = host.task("check", "/bin/sh", ".")
    |> host.with_args(["-c", "printf hello"])
  let config = local.Config(
    binary: "/absolute/path/to/target/release/fluentci-engine",
    run_root: "/tmp/fluentci-runs",
    progress: local.Auto,
  )
  let assert Ok(execution) = local.start(host.plan([task]), config)
  use completed <- promise.map(local.wait(execution))
  let assert Ok(completed) = completed
  completed.result
}
```

`start` validates the plan synchronously, then prepares a unique private run
wrapper and launches the engine. Relative workspace, task, cache and resource
paths resolve against the caller's working directory captured at `start`.
`wait` returns a Promise of a typed result with the execution directory, durable
run outcome, task statuses, exit codes, timing, log paths and cache information.
A failed or cancelled pipeline is an `Ok` report; invalid plans, launch/I/O failures
and malformed reports are separate adapter errors. Legacy reports without a run
outcome remain readable. Unknown statuses and duplicate task IDs are rejected.

Retain the execution handle to call `local.cancel(execution)`. Its Promise resolves
after cooperative SIGINT cancellation and engine closure, including child cleanup.
You can then `wait` for the cancelled report. Cancellation before launch produces a
transport error because no engine report exists. Cancellation does not impose a
hard kill deadline on the engine. Run directories and raw logs are retained;
callers own their retention policy. Live progress inherits the caller's terminal.

### Execute on a run server

`fluentci/host/client` submits the same plan to `fluentci-engine serve-runs`:

```gleam
import fluentci/host
import fluentci/host/client
import gleam/javascript/promise

pub fn main() {
  let task = host.task("check", "/bin/true", "/absolute/server/workspace")
  let config = client.Config(
    base_url: "http://127.0.0.1:8080",
    token: "replace-with-your-server-token",
    timeout_ms: 5000,
  )
  use admitted <- promise.map(client.submit_with_key(
    host.plan([task]), config, "unique-build-attempt",
  ))
  let assert Ok(state) = admitted
  state.id
}
```

Servers configured with `--max-queued` can return `Queued` before `Running`;
keep polling both states. Cancelling a queued run before execution leaves no
result report. After a hard server stop, queued runs become `Interrupted` by
default. Servers using `--resume-queued` can resume fully admitted, never-started
queued runs with the same IDs and idempotency keys. Continue polling the existing
handle; running/interrupted runs are never automatically re-executed.

Paths refer to the **server filesystem**; the adapter does not upload a local
workspace. Store the returned ID and use `get_state(id, config)` to poll until
`Succeeded`, `Failed`, `Cancelled`, or `Interrupted`. Then `get_result(id, config)`
retrieves the typed terminal report; interrupted runs may have no report.
`cancel(id, config)` requests cancellation; its acceptance does not mean cleanup
has finished. Poll state to observe completion. A result request made before
publication can return `HttpError(404, ...)`.

`submit` sends once. If a submission times out or loses its response, admission
may already have happened. Use `submit_with_key` and retry the **same plan and key**
to recover its run ID without executing twice; a changed plan returns HTTP 409.
The adapter checks the server's key acknowledgement. It never retries automatically.
Choose a new key for a new execution, and persist keys across client restarts.

The HTTP adapter accepts an HTTP(S) origin with no credentials, path, query or
fragment. Requests carry the bearer token and reject redirects. Each request has
a deadline covering headers and body reads, plus a 16 MiB decoded response limit.
Transport failures are separate from `HttpError`, malformed response `DecodeError`,
and invalid plan errors. HTTP error bodies come from the server. Promises do not
provide an independent request-abort handle; the configured timeout bounds them.
Use `get_logs(id, task_index, stream, cursor, config)` to follow raw output while
execution continues or after completion. Task indices are zero-based positions in
the submitted plan. Streams are `Stdout`, `Stderr`, `ProbeStdout`, and `ProbeStderr`.
Each response contains a `BitArray` and `next_cursor`; start at zero and persist the
returned cursor to resume without replaying already-read bytes. Responses contain
at most 1 MiB. The client rejects missing, unsafe or inconsistent cursor headers.

An empty page means no more bytes are available **yet**. Poll run state separately
to decide whether execution is terminal, then drain remaining output. A probe log
can return HTTP 404 before its first probe. Keep raw bytes when saving logs;
individual pages may split UTF-8 characters, so use a streaming decoder for text
presentation. `gleam/bit_array.to_string` is suitable only when the bytes being
decoded form a complete UTF-8 sequence.

Use `get_events(id, cursor, config)` for typed journal pages up to 4 MiB. The
`fluentci/host/journal` module defines `RunStarted`, `TaskState`, `Log`, and
`RunFinished` variants. Records include sequence numbers, elapsed time, and optional
trace/span IDs; task-state events include cache decisions, wait reasons and errors.
All task lifecycle states—including service readiness and stopping—are represented.

Start the event cursor at zero and save each page's `next_cursor`. This cursor is a
**byte offset**, distinct from an event sequence number or a raw-log cursor. Resume
with the returned offset, including after reconnecting. Empty pages may occur while
a run is active. The client checks byte counts, UTF-8, record boundaries, field types
and increasing sequence numbers within each page. Unknown event variants fail with
`DecodeError`; update the SDK when a server introduces new journal variants.
Consumers can retain the last sequence number to detect accidental replay across
pages. Journal log text is intended for display; use `get_logs` for exact bytes.

### Link server runs to a caller trace

Pass headers exported by your application's OpenTelemetry propagator explicitly:

```gleam
let context = client.TraceContext(traceparent, tracestate)
client.submit_with_context(plan, config, Some("unique-build-attempt"), context)
```

Import `Some` from `gleam/option`; use `None` for an unkeyed submission. An empty
`tracestate` omits that header. The adapter forwards only `traceparent` and
`tracestate`, rejects empty explicit parents, control characters, non-ASCII values
and headers longer than 512 bytes, and leaves trace-context semantics to the
engine's propagator. Invalid semantic context may therefore start a new trace;
use your tracing library's exported context. This does not create a caller span
or forward baggage automatically.

Configure the server's OpenTelemetry exporter to record/export run and step spans.
Journal event trace/span IDs let your UI correlate execution with those spans.
Replaying an existing idempotency key preserves the original run's trace even
when the retry carries a different context. Tests use a local OTLP HTTP/protobuf
collector and check journal correlation and exported run/step spans after server
shutdown. They do not contact an external telemetry service.

### Discover persisted runs

`history(limit, before, config)` returns `RunPage(runs, next_cursor)`, ordered
newest first by creation time with a run-ID tie-breaker. Use a limit from 1 to 200,
start with `None`, then pass `next_cursor` unchanged until it is `None`. Cursors
are opaque strings and are separate from log/event byte cursors. Restart with
`None` to discover newly submitted runs; a traversal of older pages excludes new
runs that sort above its cursor. State values can change while you traverse pages.

Each run includes its ID, typed status, optional error and optional creation time.
The client rejects oversized pages, duplicate IDs, empty continuation pages and
repeated continuation cursors. History survives a server restart; the integration
test restarts the release server, rediscovers the same run, and fetches its result.
History requests use the same bearer authentication and transport bounds as other
client calls.

The existing APIs below use the legacy engine path. Verify host plans, local
execution and server execution against a built release engine with:

```sh
FLUENTCI_ENGINE_BIN=/absolute/path/to/target/release/fluentci-engine gleam test
FLUENTCI_ENGINE_BIN=/absolute/path/to/target/release/fluentci-engine gleam test --runtime nodejs
```

Without that environment variable, serialization tests run and engine integration
is explicitly reported as skipped. Set `FLUENTCI_REQUIRE_ENGINE=1` to make a missing
engine configuration fail instead; host CI requires this on Linux and both macOS
architectures using pinned Gleam/Deno versions and Node 26. The engine fixture uses temporary directories,
checks cold/warm caching across workspaces, typed local success/failure, launch errors,
cancellation cleanup, authenticated server execution, keyed replay, live binary
output, service probe logs, typed journal events, resumable byte cursors and HTTP
error paths, trace propagation, local OTLP export, history pagination and restart
rediscovery. Tests perform no package installation.

## Legacy API quick start

```gleam
import fluentci.{dag, pipeline, set_secret}
import fluentci/pipeline.{stdout, with_exec, with_secret_variable}
import fluentci/secret.{plaintext}
import gleam/io
import gleam/javascript/array.{from_list}
import gleam/javascript/promise.{await, resolve}

pub fn main() {
  // This pipeline will print "Hello, World!" to the console
  dag()
  |> pipeline("hello-world")
  |> with_exec(from_list(["echo", "Hello, World!"]))
  |> stdout
  |> await(fn(output) {
    io.println(output)
    resolve(output)
  })

  // This pipeline will print the value of a secret to the console
  let secret = dag() |> set_secret("GITHUB", "my-github-token")
  secret
  |> plaintext
  |> await(fn(value) {
    io.println(value)
    resolve(value)
  })

  dag()
  |> pipeline("secret-demo")
  |> with_secret_variable("GITHUB", secret)
  |> with_exec(from_list(["echo", "$GITHUB"]))
  |> stdout
  |> await(fn(value) {
    io.print("Secret demo: ")
    io.println(value)
    resolve(value)
  })
}

```

Run the pipeline with the following command:

```sh
fluentci-engine run -- gleam run
```

See [fluentci-engine](https://github.com/fluentci-io/fluentci-engine) and [fluentci](https://github.com/fluentci-io/fluentci) for more information.

Further documentation can be found at <https://hexdocs.pm/fluentci>.

To verify external toolchain files, compose
`host.cache(inputs, outputs, identity) |> host.with_toolchain_files(paths)` before
attaching the cache to a task. Paths identify absolute regular-file targets.
The engine checks their contents and permissions before publishing a cached result.
The `demo` package uses the SDK from this checkout; run `cd demo && gleam build`.

## Parallel matrix jobs (unreleased)

See the [matrix guide](../../docs/matrix.md) and the `matrix` export in this SDK's
plugin example. The API expands named typed cases into native tasks and submits
one complete plan via `execute_plan`. It supports bounded concurrency, independent
case caching/logs, prerequisites, joins, failure reports and owning-run cancellation.
Use matching engine/SDK source builds; engine 0.6.0 does not contain this new import.
