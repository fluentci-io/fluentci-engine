import fluentci/client.{dag, pipeline, set_secret}
import fluentci/mise
import fluentci/pipeline.{mise, stdout, with_exec, with_secret_variable}
import fluentci/secret.{plaintext}
import gleam/io
import gleam/javascript/array.{from_list}
import gleam/javascript/promise.{await, resolve}

pub fn main() {
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
  |> with_exec(from_list(["echo $GITHUB"]))
  |> stdout
  |> await(fn(value) {
    io.print("Secret demo: ")
    io.println(value)
    resolve(value)
  })

  dag()
  |> pipeline("mise-demo")
  |> mise
  |> mise.with_workdir("./mise-demo")
  |> mise.with_exec(from_list(["mise", "--version"]))
  |> mise.with_exec(from_list(["which", "bun"]))
  |> mise.stdout
  |> await(fn(value) {
    io.print("Mise: ")
    io.println(value)
    resolve(value)
  })
}
