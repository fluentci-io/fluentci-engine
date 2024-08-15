open Dagger

let output =
  await dag
  ->Client.pipeline("demo")
  ->Client.container
  ->Container.from("alpine:latest")
  ->Container.withExec(["echo", "Hello, World!"])
  ->Container.stdout

Console.log("Output: " ++ output)
