import fluentci/client.{dag, set_secret}
import fluentci/secret.{plaintext}
import gleam/io
import gleam/javascript/promise.{await, resolve}

pub fn main() {
  let secret = dag() |> set_secret("my-secret", "my-value")
  secret
  |> plaintext
  |> await(fn(value) {
    io.debug(value)
    resolve(value)
  })
}
