import fluentci/dagger/client.{container, dag, pipeline}
import fluentci/dagger/container.{from, stdout, with_exec}
import gleam/io
import gleam/javascript/array.{from_list}
import gleam/javascript/promise.{await, resolve}

pub fn main() {
  dag()
  |> pipeline("demo")
  |> container()
  |> from("alpine:latest")
  |> with_exec(from_list(["echo", "Hello, World!"]))
  |> stdout
  |> await(fn(value) {
    io.println(value)
    resolve(value)
  })
}
