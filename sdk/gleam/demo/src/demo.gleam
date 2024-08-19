import fluentci.{dag, hermit, mise, pipeline, set_secret}
import fluentci/hermit
import fluentci/mise.{trust, with_workdir}
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

  dag()
  |> mise()
  |> with_workdir("./mise-demo")
  |> trust()
  |> mise.with_exec(from_list(["which", "bun"]))
  |> mise.stdout
  |> await(fn(value) {
    io.print("Mise demo: ")
    io.println(value)
    resolve(value)
  })

  dag()
  |> hermit()
  |> hermit.with_workdir("./hermit-demo")
  |> hermit.with_exec(from_list(["which", "jq"]))
  |> hermit.stdout
  |> await(fn(value) {
    io.print("Hermit demo: ")
    io.println(value)
    resolve(value)
  })
}
