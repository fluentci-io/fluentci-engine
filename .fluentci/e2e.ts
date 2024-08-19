import { dag } from "../sdk/typescript/mod.ts";

const plugins = [
  "archive",
  "chmod",
  "devbox",
  "flox",
  "git",
  "hash",
  "hermit",
  "http",
  "mise",
  "nix",
  "pixi",
  "pkgx",
  "proto",
];

await dag
  .pipeline("plugin-e2e")
  .withExec(["rustup", "target", "add", "wasm32-unknown-unknown"])
  .withWorkdir("../examples")
  .withExec([
    `cargo build ${plugins
      .map((item) => `-p ${item}`)
      .join(" ")} --release --target wasm32-unknown-unknown`,
  ])
  .withWorkdir("../fixtures")
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/flox.wasm",
    "exec which jq",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/archive.wasm",
    "tar_czvf flox-demo/.flox",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/hash.wasm",
    "md5 flox-demo/.flox.tar.gz",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/hash.wasm",
    "sha256 flox-demo/.flox.tar.gz",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/nix.wasm",
    "exec which deno",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/devbox.wasm",
    "exec which gh",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/git.wasm",
    "git_tree https://github.com/tsirysndr/me",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/mise.wasm",
    "exec which bun",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/pixi.wasm",
    "exec which php",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/pkgx.wasm",
    "exec which deno",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/hermit.wasm",
    "exec which jq",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/chmod.wasm",
    "chmod a+x hello.sh",
  ])
  .withExec([
    "fluentci-engine",
    "call",
    "-m",
    "../target/wasm32-unknown-unknown/release/http.wasm",
    "get https://fluentci.io",
  ])
  .stdout();

const demo = await dag
  .pipeline("typescript e2e")
  .withExec([
    "fluentci-engine run -- deno run -A ../sdk/typescript/demo/main.ts",
  ])
  .stdout();

console.log(demo);

const queries = [
  "zip",
  "tar-czvf",
  "unzip",
  "tar-xzvf",
  "md5",
  "sha256",
  "git",
  "directory",
  "envhub",
  "mise",
  "pixi",
  "pkgx",
  "nix",
  "flox",
  "devenv",
  "devbox",
  "proto",
];

const engine = dag
  .pipeline("fluentci-service")
  .withExec(["fluentci-engine", "serve"])
  .asService("fluentci-engine");

let query = dag
  .pkgx()
  .withService(engine)
  .withPackages(["httpie"])
  .waitOn(6880);

for (const item of queries) {
  query = query.withExec([
    `http POST http://localhost:6880/graphql Content-Type:application/json query="$(cat ${item}.graphql)" --ignore-stdin --pretty format`,
  ]);
}

console.log(await query.stdout());
