import assert from "node:assert/strict";
import { test } from "node:test";
import * as S from "./output/Sccache/index.js";
import { compile } from "./output/FluentCI.Host.Plan/index.js";

function right(result) {
  assert.equal(result.constructor.name, "Right", result.value0);
  return result.value0;
}
const config = {
  project: "/tmp/project ' with spaces", run: "/tmp/run ' with spaces",
  install: "/tmp/bin with spaces", cache: "/tmp/cache with spaces", path: "/usr/bin:/bin",
};
const plan = value => JSON.parse(right(compile(value)));

test("selects verified releases and rejects unsupported provisioning targets", () => {
  for (const os of ["linux", "macos"]) for (const arch of ["x86_64", "aarch64"]) {
    const artifact = right(S.artifactFor(os)(arch));
    assert.match(artifact.checksum, /^[a-f0-9]{64}$/);
    assert.match(artifact.target, os === "linux" ? /linux-musl$/ : /apple-darwin$/);
  }
  assert.equal(S.artifactFor("netbsd")("x86_64").constructor.name, "Left");
});

test("validates input and keeps Cargo arguments literal", () => {
  const args = ["--package", "literal; $(touch unexpected)"];
  assert.deepEqual(right(S.validateArguments([config.project, config.run, ...args])).cargo, args);
  for (const input of [[], ["."], ["", "run"], [".", "run", "--profile=dev"]])
    assert.equal(S.validateArguments(input).constructor.name, "Left");
  const document = plan(S.buildPlan(config)("/tmp/sccache")("/tmp/test.sock")(args));
  assert.equal(document.inherit_env, true);
  assert.equal(document.jobs, 1);
  assert.equal(document.max_services, 1);
  const [server, build, stats] = document.tasks;
  assert.deepEqual(server.args, []);
  assert.equal(server.program, "/tmp/sccache");
  assert.equal(server.env.SCCACHE_START_SERVER, "1");
  assert.equal(server.env.SCCACHE_NO_DAEMON, "1");
  assert.deepEqual(server.resources, ["sccache-storage:" + config.cache]);
  assert.equal(server.service.readiness.program, "/bin/test");
  assert.deepEqual(build.args, ["build", "--release", "--locked", ...args]);
  assert.equal(build.env.RUSTC_WRAPPER, "/tmp/sccache");
  assert.equal(build.env.SCCACHE_START_SERVER, undefined);
  assert.deepEqual(stats.needs, ["compiler-cache", "build"]);
  assert.ok(document.tasks.every(task => !/\/(?:ba)?sh$/.test(task.program)));
});

test("download and installation plans use native tools with literal paths", () => {
  const artifact = right(S.artifactFor("linux")("x86_64"));
  const stage = "/tmp/stage ' $(touch unexpected)";
  const download = plan(S.downloadPlan(config)(artifact)(stage)).tasks[0];
  assert.equal(download.program, "/usr/bin/curl");
  assert.equal(download.args.at(-1), stage + "/archive.tar.gz");
  const [extract, mode, install] = plan(S.installPlan(config)(artifact)(stage)).tasks;
  assert.equal(extract.program, "/usr/bin/tar");
  assert.equal(extract.args[1], stage + "/archive.tar.gz");
  assert.deepEqual(mode.needs, ["extract"]);
  assert.deepEqual(install.needs, ["executable"]);
  assert.equal(install.args.at(-1), config.install + "/sccache");
});
