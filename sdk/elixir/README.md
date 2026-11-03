# FluentCI Elixir host plans

An initial Elixir adapter for protocol-v1 host execution plans. Values are immutable
and builders take the value first for `|>` composition. Building and compiling
perform no I/O. This package has no third-party dependencies and requires Elixir
1.18+ for its [built-in JSON module](https://hexdocs.pm/elixir/JSON.html).
It is available from this repository; it has not been published to Hex.

Add a local path dependency to an Elixir project's `mix.exs`:

```elixir
{:fluentci, path: "../fluentci-engine/sdk/elixir"}
```

```elixir
alias FluentCI.Host, as: H

build =
  H.task("build", "/bin/sh", ".")
  |> H.args(["-c", "/bin/cat input > output"])
  |> H.timeout_ms(30_000)
  |> H.cache(["input"], ["output"], "fixture-v1", relocatable: true)

inspect_output =
  H.task("inspect", "/bin/cat", ".")
  |> H.args(["output"])
  |> H.needs(["build"])

plan =
  H.plan([build, inspect_output])
  |> H.jobs(2)
  |> H.workspace("/absolute/path/to/project", ["input"])
  |> H.cache_dir("/absolute/path/to/cache")

{:ok, json} = H.compile(plan)
# Explicit effect boundary: writing or submitting the resulting JSON is the caller's job.
File.write!("pipeline.json", json)
```

Run the plan with the release CLI, using a fresh run directory:

```sh
/path/to/fluentci-engine/target/release/fluentci-engine execute pipeline.json --run-dir ./run-1 --progress tui
```

Plans always use `inherit_env: false`. Use `H.env(task, %{"NAME" => "value"})`
for explicit child environment values; no shell interpolation is performed by
builders. A shell only runs if explicitly selected as the program. Outputs from a
copied workspace stay under the run directory; they are not copied back to source.
Relocatable caching is an explicit promise that outputs and logs do not depend on
the physical working-directory path. See the [host executor contract](../../docs/host-executor.md).

Plan setters include `jobs`, `max_services`, `fail_fast`, `cache_dir`, `resource_dir`,
`workspace`, and `add_task`. Task setters include `args`, `needs`, `env`, `resources`,
`shared_resources`, `timeout_ms`, `resource_wait_timeout_ms`, and `package_environment`.
List/map setters replace that field rather than mutating or appending to it.
`compile/1` returns `{:ok, json}` or `{:error, reason}`. Errors identify invalid field
paths, duplicate IDs, missing dependencies, cycles, or incompatible task modes.
Validation errors do not include environment values. Filesystem validity, cache
path overlap, resource contracts, provider availability and server admission limits
are also validated by the engine when executing/submitting the plan.

Services use the same explicit argv model:

```elixir
ready = H.probe("/absolute/path/to/pg_isready", [], interval_ms: 100, timeout_ms: 1000)
health = H.probe("/absolute/path/to/pg_isready", [], interval_ms: 500, timeout_ms: 1000)

database =
  H.task("database", "/absolute/path/to/postgres", "/absolute/data-directory")
  |> H.service(ready, health: health, startup_timeout_ms: 30_000,
       shutdown_timeout_ms: 5000, failure_threshold: 3)

# Consumers declare H.needs(consumer, ["database"]). The engine waits for readiness,
# runs health checks, and stops the service after its transitive consumers finish.
```

Package providers are `:homebrew`, `:macports`, `:macports_rust`, `:freebsd`, `:netbsd`,
and `:openbsd`. `H.provider(kind, executable: "/explicit/path")` overrides the
manager executable. The Rust MacPorts backend selects macports-rs. Homebrew is
limited to macOS ARM64/Linux x86_64; MacPorts is macOS-only; BSD providers require
the matching host OS. The engine applies these platform gates at execution time.

```elixir
ports = H.provider(:macports_rust)
install = H.install("packages", ports, ["jq"], "/absolute/project")
consumer = H.task("jq", "jq", "/absolute/project")
  |> H.args(["--version"])
  |> H.needs(["packages"])
  |> H.package_environment([ports])
{:ok, json} = H.plan([install, consumer]) |> H.compile()
```

Installation is a separate uncached step with a prefix lease. No package manager
is installed or bootstrapped by the SDK, and compilation never installs packages.
The generated JSON can be passed to the existing CLI or authenticated run API.
A managed local runner is also available; an Elixir HTTP client and event decoder
remain work for subsequent milestones.

## Managed local execution

```elixir
alias FluentCI.Host.Local

{:ok, execution} = Local.start(plan,
  binary: "/path/to/fluentci-engine/target/release/fluentci-engine",
  cwd: "/absolute/path/to/project",
  run_root: "/absolute/path/to/run-storage")

try do
  case Local.await(execution) do
    {:ok, %Local.Report{result: result, exit_status: status}} ->
      IO.inspect({result.outcome, status})
    {:error, %Local.Error{} = error} ->
      IO.inspect(error.kind)
  end
after
  Local.close(execution)
end
```

`start/2` compiles before creating run files and starts a temporary OTP-supervised
worker. The `binary` option defaults to `fluentci-engine` on PATH; `cwd` defaults to
the caller's current directory and `run_root` to the system temporary directory.
A Unix `kill` executable is resolved once from PATH (with `/bin/kill` as fallback)
for cancellation; a missing signal utility is rejected before starting the engine.
Relative task directories (without copied workspaces), workspace source, cache
and resource directories resolve against this captured cwd, not the wrapper directory.
Each execution gets a fresh mode-0700 wrapper and mode-0600 plan file. `run_directory`
identifies the engine's execution directory inside it. BEAM-global cwd/env are never
modified. Optional `env: %{"NAME" => "value", "REMOVE_ME" => nil}` overrides only
the engine process environment; task environments remain explicit plan data.

`await/2` returns a typed `Local.Report` only after engine exit. Pipeline failure
or cancellation is still `{:ok, report}`; startup, I/O and decoding failures return
`{:error, %Local.Error{}}`. Reports must be regular files, at most 16 MiB, and match
the submitted task IDs in order. CLI exit status is retained separately from the
pipeline outcome. Repeated awaits return the same completed result.

`cancel/2` sends SIGINT and waits for the engine's process/service cleanup and
terminal result. Late cancellation preserves an already completed result. An early
cancellation before engine startup may have no result file and returns an error
instead of inventing a report. `await/2`, `cancel/2`, and `close/2` accept a millisecond
call timeout (default `:infinity`); timing out the call does not kill the execution
worker. An already requested cancellation continues. Only one pending wait is kept
per caller; repeated timed-out polls replace it, and dead callers are removed.
Use task/service deadlines to
bound work and call `await` again to inspect its eventual result.

The process calling `start` owns the execution. Its death initiates cancellation,
including normal exit: do not return a handle from a short-lived Task unless that
cancellation is intended. Temporary supervisor children never automatically replay
an execution. `close/2` cancels and waits if needed, then releases the handle while
retaining all run files. Use it to release completed handles owned by long-lived
processes. Ordinary OTP worker termination also requests cleanup. Cleanup is
cooperative and has no forced-kill deadline; a stuck engine can delay shutdown.
VM crashes, forced worker kills, or engine crashes do not guarantee orphan cleanup.
This does not replace the engine's still-unfinished hard-crash supervision.

`Local.output/1` polls the most recent 64 KiB of merged CLI diagnostic output while
running or after completion. It returns binary bytes and may start mid-UTF-8; it is
not a complete or resumable log stream. Exact task/probe bytes remain in report log
files. Errors include the run directory, exit status when available, and a bounded
diagnostic tail; that tail may contain task output and should be handled accordingly.

## Terminal result decoding

```elixir
alias FluentCI.Host.Result

json = File.read!("run-1/result.json") # Caller-controlled I/O boundary.
case Result.decode(json) do
  {:ok, %Result{outcome: :succeeded}} -> :ok
  {:ok, %Result{} = report} -> {:pipeline_result, report}
  {:error, %Result.Error{path: path, reason: reason}} -> {:invalid_report, path, reason}
end
```

A failed or cancelled pipeline is a successfully decoded report, not a decoding
error. `Result.from_map/1` accepts already parsed string-keyed JSON objects.
`Result.succeeded?/1` checks that every task succeeded. No decoder reads log files
or starts processes. Paths in a remote report refer to the server filesystem.

Run outcomes are `:succeeded`, `:failed`, or `:cancelled`. Missing/null outcomes from
older engines remain `nil`; task cancellation alone does not imply external run
cancellation because fail-fast failures can cancel sibling tasks too. Task statuses
are `:succeeded`, `:failed`, `:skipped`, `:cancelled`, and `:timed_out`.

`Result.Task` retains ID, exit code, error, duration, stdout/stderr paths, cache
information, optional probe-log paths, and resource wait diagnostics. Cache outcome
strings remain open to new engine variants; component digests are retained in a
string-keyed map. Duration integers preserve precision through the engine's u128
range. Unknown fields are ignored for forward compatibility. Unknown/nonterminal
statuses, duplicate task IDs, malformed fields, and contradictory outcomes are
rejected with field paths. External values are never converted into atoms, and
decoding errors contain no raw report values.

## Validation

From the engine root, then the SDK directory:

```sh
cargo build --release -p fluentci-engine
cd sdk/elixir
FLUENTCI_REQUIRE_ENGINE=1 mix test --warnings-as-errors
mix format --check-formatted
```

`FLUENTCI_ENGINE_BIN` overrides the engine path. Without the binary, engine tests
are explicitly skipped unless `FLUENTCI_REQUIRE_ENGINE=1` makes that a failure.
Tests cover pure composition, dependency graphs, invalid wire values, every provider
encoding, and actual release-engine workspace/cache, failed-dependency, and service
execution, plus actual timeout results. Decoder tests cover terminal variants,
legacy reports, malformed fields, outcome consistency, duplicate IDs, and integer
precision. Local-runner tests cover concurrent cwd/env isolation, repeatable results,
live output, cancellation and descendant cleanup, owner death, OTP termination,
service/health/consumer cancellation with lease reuse, early cancellation, private
files, malformed or oversized reports, bounded waiters, and absence of automatic
replay after handle termination. Package-manager encoding tests perform
no installation. The workflow
includes these tests on Ubuntu with Elixir 1.18.4/OTP 27.3.4; local validation also
runs on macOS ARM64. A configured CI job is not evidence of a completed remote run.
