open FluentCI
open FluentCI.Deno

let secret = dag->Client.setSecret(~name="GITHUB", ~value="my-github-token")

Console.log(await secret->Secret.plaintext)

let secretDemo =
  await dag
  ->Client.pipeline(~name="secret-demo")
  ->Pipeline.withSecretVariable(~name="GITHUB", ~secret)
  ->Pipeline.withExec(["echo $GITHUB"])
  ->Pipeline.stdout

Console.log("Secret demo: " ++ secretDemo)

let cache = dag->Client.cache(~key="pixi")

Console.log("cacheId: " ++ await cache->Cache.id)

let ping =
  dag
  ->Client.pipeline(~name="demo")
  ->Pipeline.nix({impure: true})
  ->Nix.withWorkdir(~path="nix-demo")
  ->Nix.withExec(["ping", "fluentci.io"])
  ->Nix.asService(~name="ping_service")

Console.log("Ping service: " ++ await ping->Service.id)

let pingGithub =
  dag
  ->Client.pipeline(~name="ping-github")
  ->Pipeline.pkgx
  ->Pkgx.withPackages(["ping"])
  ->Pkgx.withExec(["ping", "github.com"])
  ->Pkgx.asService(~name="ping_github")

let stdout =
  await dag
  ->Client.pipeline(~name="demo")
  ->Pipeline.pkgx
  ->Pkgx.withPackages(["ping"])
  ->Pkgx.withService(ping)
  ->Pkgx.withService(pingGithub)
  ->Pkgx.withExec(["sleep", "15"])
  ->Pkgx.withExec(["ls", "-ltr", ".fluentci"])
  ->Pkgx.withExec(["cat", ".fluentci/process-compose.yaml"])
  ->Pkgx.stdout

Console.log("Stdout: " ++ stdout)

let mise =
  await dag
  ->Client.pipeline(~name="mise-demo")
  ->Pipeline.mise
  ->Mise.withWorkdir(~path="./mise-demo")
  ->Mise.trust
  ->Mise.withExec(["mise", "--version"])
  ->Mise.withExec(["which", "bun"])
  ->Mise.stdout

Console.log("Mise: " ++ mise)

let git =
  await dag
  ->Client.git(~url="https://github.com/tsirysndr/me")
  ->Git.branch("main")
  ->Git.tree
  ->Directory.withExec(["pwd"])
  ->Directory.stdout

Console.log("Git: " ++ git)

let gitEntries =
  await dag
  ->Client.git(~url="https://github.com/tsirysndr/me")
  ->Git.branch("main")
  ->Git.tree
  ->Directory.entries

Console.log("Git entries: ")
Console.log(gitEntries)

let dir = await dag->Client.directory(~path=".")->Directory.entries

Console.log("Directory entries at '.': ")
Console.log(dir)

let hermit =
  await dag
  ->Client.hermit
  ->Hermit.withWorkdir(~path="./hermit-demo")
  ->Hermit.withExec(["which", "jq"])
  ->Hermit.stdout

Console.log("Hermit: ")
Console.log(hermit)
