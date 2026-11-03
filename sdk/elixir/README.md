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
This milestone does not include an Elixir-managed local process runner, HTTP run
client, cancellation handle, or typed result/event decoders. The generated JSON can
be passed to the existing CLI or authenticated run API; those SDK APIs remain work
for subsequent milestones.

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
execution. Package-manager encoding tests perform no installation. The workflow
includes these tests on Ubuntu with Elixir 1.18.4/OTP 27.3.4; local validation also
runs on macOS ARM64. A configured CI job is not evidence of a completed remote run.
