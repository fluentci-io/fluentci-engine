import { dag } from "../src/client.gen.ts";

async function main() {
  Deno.env.set("FLUENTCI_SESSION_PORT", "6880");
  Deno.env.set("FLUENTCI_SESSION_TOKEN", "token");

  const demo = await dag
    .pipeline("demo")
    .withWorkdir("./")
    .withExec(["echo", "hello"])
    .withExec(["echo", "hello world"])
    .withExec(["echo", "hello again"])
    .stdout();

  console.log(demo);

  const envhub = await dag
    .pipeline("envhub-demo")
    .envhub()
    .use("github:tsirysndr/dotfiles-example")
    .withExec(["envhub", "--version"])
    .withExec(["which", "hello"])
    .stdout();

  console.log(envhub);

  const pixi = await dag
    .pipeline("pixi-demo")
    .pixi()
    .withWorkdir("./pixi-demo")
    .withExec(["pixi", "--version"])
    .withExec(["which", "php"])
    .stdout();

  console.log(pixi);

  const zip = await dag.directory("./pixi-demo").zip().path();

  console.log(zip);

  const tarCzvf = await dag.directory("./pixi-demo").tarCzvf().path();

  console.log(tarCzvf);

  const unzip = await dag
    .file("./pixi-demo.zip")
    .unzip("./pixi-demo-output-zip")
    .entries();

  console.log(unzip);

  const tarXzvf = await dag
    .file("./pixi-demo.tar.gz")
    .unzip("./pixi-demo-output-tar")
    .entries();

  console.log(tarXzvf);

  const md5 = await dag.file("./pixi-demo.tar.gz").md5();

  console.log(md5);

  const sha256 = await dag.file("./pixi-demo.tar.gz").sha256();

  console.log(sha256);

  const mise = await dag
    .pipeline("mise-demo")
    .mise()
    .withWorkdir("./mise-demo")
    .withExec(["mise", "--version"])
    .withExec(["which", "bun"])
    .stdout();

  console.log(mise);

  const pkgx = await dag
    .pipeline("pkgx-demo")
    .pkgx()
    .withWorkdir("./pkgx-demo")
    .withExec(["pkgx", "--version"])
    .withExec(["which", "deno"])
    .stdout();

  console.log(pkgx);

  const git = await dag
    .git("https://github.com/tsirysndr/me")
    .branch("main")
    .tree()
    .withExec(["pwd"])
    .stdout();
  console.log(git);

  const gitEntries = await dag
    .git("https://github.com/tsirysndr/me")
    .branch("main")
    .tree()
    .entries();

  console.log(gitEntries);

  const dir = await dag.directory(".").entries();

  console.log(dir);

  const nix = await dag
    .pipeline("nix-demo")
    .nix()
    .withWorkdir("./nix-demo")
    .withExec(["nix", "--version"])
    .withExec(["which", "deno"])
    .stdout();

  console.log(nix);

  const devbox = await dag
    .pipeline("devbox-demo")
    .devbox()
    .withWorkdir("./devbox-demo")
    .withExec(["devbox", "version"])
    .withExec(["which", "jq"])
    .stdout();

  console.log(devbox);

  const flox = await dag
    .pipeline("flox-demo")
    .flox()
    .withWorkdir("./flox-demo")
    .withExec(["flox", "--version"])
    .withExec(["which", "jq"])
    .stdout();

  console.log(flox);
}

// Learn more at https://deno.land/manual/examples/module_metadata#concepts
if (import.meta.main) {
  await main();
}
