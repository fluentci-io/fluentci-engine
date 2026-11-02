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

A typed HTTP adapter is still outstanding. The existing APIs below use the legacy
engine path. Verify host plans and local execution against a built release engine with:

```sh
FLUENTCI_ENGINE_BIN=/absolute/path/to/target/release/fluentci-engine gleam test
FLUENTCI_ENGINE_BIN=/absolute/path/to/target/release/fluentci-engine gleam test --runtime nodejs
```

Without that environment variable, serialization tests run and engine integration
is explicitly reported as skipped. Set `FLUENTCI_REQUIRE_ENGINE=1` to make a missing
engine configuration fail instead; host CI requires this on Linux and both macOS
architectures using pinned Gleam/Deno versions and Node 26. The engine fixture uses temporary directories,
checks cold/warm caching across workspaces, typed local success/failure, launch errors,
and cancellation cleanup. Tests perform no package installation.

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
