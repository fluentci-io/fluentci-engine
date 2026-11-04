import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { delimiter, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = fileURLToPath(new URL("../..", import.meta.url));
const cli = process.env.FLUENTCI_BIN || "fluentci";

function fixture(t) {
  mkdirSync(join(root, "target"), { recursive: true });
  const directory = mkdtempSync(join(root, "target/ci-module-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  mkdirSync(join(directory, "bin"));
  mkdirSync(join(directory, "fixtures"));
  const log = join(directory, "commands.jsonl");
  const script = `#!/usr/bin/env node
import { appendFileSync } from "node:fs";
import { basename } from "node:path";
const command = basename(process.argv[1]);
appendFileSync(process.env.CI_TEST_LOG, JSON.stringify({command,args:process.argv.slice(2),cwd:process.cwd()}) + "\\n");
if (process.env.CI_TEST_FAIL === command + ":" + process.argv[2]) process.exit(7);
`;
  function executable(path, content = script) {
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, content);
    chmodSync(path, 0o755);
  }
  for (const name of ["cargo", "rustup", "gh", "aws", "nix"]) executable(join(directory, "bin", name));
  executable(join(directory, "target/release/fluentci-engine"));
  function run(module, fn, extra = {}) {
    const env = {...process.env, PATH: join(directory, "bin") + delimiter + process.env.PATH, CI_TEST_LOG: log, ...extra};
    delete env.FLUENTCI_PROJECT_ID;
    const output = spawnSync(cli, ["--check-update=false", "--database", join(directory,"db"), "run", "--work-dir", directory, module, fn], {cwd:directory,env,encoding:"utf8",timeout:120000});
    if (output.error) throw output.error;
    return output;
  }
  const commands = () => existsSync(log) ? readFileSync(log,"utf8").trim().split("\n").map(JSON.parse) : [];
  return {directory,run,commands,executable};
}

function success(output) { assert.equal(output.status,0,output.stderr + output.stdout); }

test("Rust tasks use native release builds and locked dependencies", t => {
  const f=fixture(t);
  success(f.run("rust","test"));
  success(f.run("rust","build"));
  success(f.run("rust","clippy"));
  const commands=f.commands();
  assert.deepEqual(commands.map(({command,args})=>[command,...args]),[
    ["cargo","test","--release","--locked","-p","fluentci-engine","-p","fluentci-executor","-p","fluentci-core","-p","fluentci-server","--lib","--tests"],
    ["cargo","build","--release","--locked","-p","fluentci-engine"],
    ["cargo","clippy","--release","--locked","-p","fluentci-executor","--all-targets","--","-D","warnings"],
  ]);
  assert.ok(commands.every(c=>c.cwd===f.directory));
});

test("the provider Wasm build and every former provider invocation are preserved", t => {
  const f=fixture(t);
  success(f.run("e2e","buildWasm"));
  success(f.run("e2e","providers"));
  const commands=f.commands();
  assert.deepEqual(commands[0].args,["target","add","wasm32-unknown-unknown"]);
  assert.deepEqual(commands[1].args.slice(0,5),["build","--release","--locked","--target","wasm32-unknown-unknown"]);
  assert.equal(commands.slice(2).length,13);
  for (const command of commands.slice(2)) {
    assert.equal(command.command,"fluentci-engine");
    assert.equal(command.cwd,join(f.directory,"fixtures"));
    assert.deepEqual(command.args.slice(0,2),["call","-m"]);
  }
  assert.equal(commands[2].args[2],"../target/wasm32-unknown-unknown/release/flox.wasm");
  assert.equal(commands[2].args[3],"exec which jq");
  assert.equal(commands.at(-1).args[3],"get https://fluentci.io");
});

test("the TypeScript SDK runs from fixtures so service and provider paths resolve", t => {
  const f=fixture(t);
  success(f.run("e2e","typescript"));
  const [command] = f.commands();
  assert.equal(command.cwd, join(f.directory, "fixtures"));
  assert.deepEqual(command.args,["run","--","deno","run","-A","../sdk/typescript/demo/main.ts"]);
  assert.ok(existsSync(join(root, "fixtures", command.args.at(-1))));
  for (const path of ["service-demo/server.ts", "service-demo/client.ts", "hermit-demo", "pixi-demo", "flox-demo", "mise-demo", "pkgx-demo", "nix-demo", "devbox-demo"]) {
    assert.ok(existsSync(join(root, "fixtures", path)), `missing SDK demo fixture: ${path}`);
  }
});

test("native release recipes build, package, checksum, and upload exact assets", t => {
  const f=fixture(t), env={TARGET:"aarch64-apple-darwin",RELEASE_VERSION:"v0.6.1-test",ACCOUNT_ID:"fixture-account"};
  f.executable(join(f.directory,"target",env.TARGET,"release/fluentci-engine"));
  success(f.run("release","build",env));
  assert.deepEqual(f.commands().slice(0,3).map(({command,args})=>[command,...args]),[
    ["rustup","target","add",env.TARGET],
    ["cargo","build","--release","--locked","-p","fluentci-engine","--target",env.TARGET],
    ["fluentci-engine","--version"],
  ]);
  success(f.run("release","package",env));
  const filename=`fluentci-engine_${env.RELEASE_VERSION}_${env.TARGET}.tar.gz`;
  const archive=join(f.directory,filename);
  const digest=createHash("sha256").update(readFileSync(archive)).digest("hex");
  assert.equal(readFileSync(archive+".sha256","utf8").trim(),`${digest}  ${filename}`);
  const listing=spawnSync("tar",["tzf",archive],{encoding:"utf8"});
  assert.equal(listing.status,0,listing.stderr);
  assert.equal(listing.stdout.trim(),"fluentci-engine");
  success(f.run("release","upload",env));
  success(f.run("release","r2",env));
  const commands=f.commands();
  assert.deepEqual(commands.at(-3).args,["release","upload",env.RELEASE_VERSION,filename,filename+".sha256","--clobber"]);
  for (const command of commands.slice(-2)) {
    assert.equal(command.command,"aws");
    assert.equal(command.args[3],`s3://assets/fluentci-engine/${env.RELEASE_VERSION}/${command.args[2]}`);
    assert.equal(command.args[5],"https://fixture-account.r2.cloudflarestorage.com");
  }
});

test("a failed release build stops before binary checks", t => {
  const f=fixture(t);
  assert.notEqual(f.run("release","build",{TARGET:"x86_64-unknown-linux-gnu",CI_TEST_FAIL:"cargo:build"}).status,0);
  assert.deepEqual(f.commands().map(c=>c.command),["rustup","cargo"]);
});

test("the portable release module runs without source compilation", t => {
  const f=fixture(t);
  const module=join(root,".fluentci/release/dist/plugin.wasm");
  success(f.run(module,"upload",{TARGET:"x86_64-unknown-freebsd",RELEASE_VERSION:"v0.6.1-test"}));
  assert.equal(f.commands()[0].args[2],"v0.6.1-test");
});
