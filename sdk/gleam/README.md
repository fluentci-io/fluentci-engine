# FluentCI Gleam SDK

[![Package Version](https://img.shields.io/hexpm/v/fluentci)](https://hex.pm/packages/fluentci)
[![Hex Docs](https://img.shields.io/badge/hex-docs-ffaff3)](https://hexdocs.pm/fluentci/)

The FluentCI Gleam SDK provides a set of APIs to interact with FluentCI in Gleam, it allows you to write CI/CD pipelines in Gleam.

## 🚚 Installation

```sh
gleam add fluentci
```

## 🚀 Quick Start

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

See [fluentci-engine](https://github.com/fluentci-io/fluentci-engine) and [fluentci](https://github.com/fluentci-io/fluenti) for more information.

Further documentation can be found at <https://hexdocs.pm/fluentci>.
