# FluentCI TypeScript

![deno compatibility](https://shield.deno.dev/deno/^1.41)
[![](https://jsr.io/badges/@fluentci/sdk)](https://jsr.io/@fluentci/sdk)
[![ci](https://github.com/fluentci-io/fluentci-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/fluentci-io/fluentci-engine/actions/workflows/ci.yml)
[![discord](https://img.shields.io/discord/1132020671262773358?label=discord&logo=discord&color=5865F2)](https://discord.gg/V4U6dPskKc)

The FluentCI TypeScript SDK contains everything you need to develop CI/CD pipelines in TypeScript or Javascript, and run them on [Nix](https://nixos.org) environments.

## Quick Start

Clone the [FluentCI Engine](https://github.com/fluentci-io/fluentci-engine) repository and start the engine at the fixtures directory:

```bash
git clone https://github.com/fluentci-io/fluentci-engine
cd fluentci-engine/fixtures
# you can get this binary from the releases page
# start the engine
fluentci-engine serve
```

Now you can save the following code to a file called `main.ts` and run it with `deno run -A main.ts`:

```typescript
import { dag } from "jsr:@fluentci/sdk";

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


  const nix = await dag
    .pipeline("nix-demo")
    .nix()
    .withWorkdir("./nix-demo")
    .withExec(["nix", "--version"])
    .withExec(["which", "deno"])
    .stdout();

  console.log(nix);

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
```