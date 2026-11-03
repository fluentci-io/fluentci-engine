import { dag } from "../src/client.gen.ts";

async function main() {
  if (!Deno.env.has("FLUENTCI_SESSION_PORT")) {
    Deno.env.set("FLUENTCI_SESSION_PORT", "6880");
    Deno.env.set("FLUENTCI_SESSION_TOKEN", "token");
  }

  const secret = dag.setSecret("DEMO", "12345");

  console.log(await secret.plaintext());

  const secretDemo = await dag
    .pipeline("secret-demo")
    .withSecretVariable("DEMO", secret)
    .withExec(["echo", "$DEMO"])
    .stdout();

  console.log("secretDemo: ", secretDemo);

  const cache = dag.cache("pixi");

  console.log("cacheId: ", await cache.id());

  const serviceDirectory = await Deno.makeTempDir({ prefix: "fluentci-services-" });
  const portFiles = [0, 1].map((index) => `${serviceDirectory}/service-${index}.port`);
  const quote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;
  try {
    const services = portFiles.map((portFile, index) =>
      dag.pipeline(`service-${index}`)
        .withExec(["deno", "run", "-A", "service-demo/server.ts", `service-${index}`, quote(portFile)])
        .asService(`http_service_${index}`)
    );
    console.log(await dag.pipeline("services-demo")
      .withService(services[0])
      .withService(services[1])
      .withExec(["deno", "run", "-A", "service-demo/client.ts", ...portFiles.map(quote)])
      .stdout());
  } finally {
    await Deno.remove(serviceDirectory, { recursive: true });
  }

  const demo = await dag
    .pipeline("demo")
    .withWorkdir("./")
    .withExec(["echo", "hello"])
    .withExec(["echo", "hello world"])
    .withExec(["echo", "hello again"])
    .stdout();

  console.log(demo);

  const hermit = await dag
    .hermit()
    .withWorkdir("./hermit-demo")
    .withPackages(["jq"])
    .withExec(["which", "jq"])
    .stdout();

  console.log(hermit);

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
    .withCache("./.pixi", cache)
    .withExec(["pixi", "--version"])
    .withExec(["which", "php"])
    .stdout();

  console.log(pixi);

  const zip = await dag.directory("./flox-demo").zip().path();

  console.log(zip);

  const tarCzvf = await dag.directory("./flox-demo").tarCzvf().path();

  console.log(tarCzvf);

  const unzip = await dag
    .file("./flox-demo.zip")
    .unzip("./flox-demo-output-zip")
    .entries();

  console.log(unzip);

  const tarXzvf = await dag
    .file("./flox-demo.tar.gz")
    .tarXzvf("./flox-demo-output-tar")
    .entries();

  console.log(tarXzvf);

  const md5 = await dag.file("./flox-demo.tar.gz").md5();

  console.log(md5);

  const sha256 = await dag.file("./flox-demo.tar.gz").sha256();

  console.log(sha256);

  /*
  await dag
    .pipeline("clean")
    .withWorkdir("./")
    .withExec([
      "rm",
      "-rf",
      "flox-demo-output-zip",
      "flox-demo-output-tar",
      "flox-demo.zip",
      "flox-demo.tar.gz",
    ])
    .stdout();
*/

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
    .nix({
      impure: true,
    })
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
