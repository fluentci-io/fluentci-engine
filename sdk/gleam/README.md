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

This module targets JavaScript and adds no dependencies. It is a plan compiler,
not yet a typed HTTP or local execution adapter. The existing APIs below still use
the legacy engine path. Verify host plans against a built release engine with:

```sh
FLUENTCI_ENGINE_BIN=/absolute/path/to/target/release/fluentci-engine gleam test
FLUENTCI_ENGINE_BIN=/absolute/path/to/target/release/fluentci-engine gleam test --runtime nodejs
```

Without that environment variable, serialization tests run and engine integration
is explicitly reported as skipped. The engine fixture uses temporary directories,
checks cold/warm caching across workspaces, and performs no package installation.

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
