open FluentCI

let secret = dag->Client.setSecret(~name="GITHUB", ~value="my-github-token")

Console.log(await secret->Secret.plaintext)

let secretDemo =
  await dag
  ->Client.pipeline(~name="secret-demo")
  ->Pipeline.withSecretVariable(~name="GITHUB", ~secret)
  ->Pipeline.withExec(["echo $GITHUB"])
  ->Pipeline.stdout

Console.log("Secret demo: " ++ secretDemo)
