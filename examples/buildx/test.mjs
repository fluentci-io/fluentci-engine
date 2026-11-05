import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import * as C from "./output/Buildx.Commands/index.js";
import { compile } from "./output/FluentCI.Host.Plan/index.js";

const snapshot = JSON.parse(readFileSync(new URL("buildx-options.json", import.meta.url)));
const schema = JSON.parse(readFileSync(new URL("../../schema/generated/buildx.json", import.meta.url)));
const camel = value => value.replace(/-([a-z])/g, (_, character) => character.toUpperCase());
const command = path => path.length ? path.join("_").replaceAll("-", "_") : "buildx";
const pure = path => path.length ? camel(path.join("-")) : "buildx";
const title = name => name[0].toUpperCase() + name.slice(1);
const right = result => {
  assert.equal(result.constructor.name, "Right", result.value0);
  return result.value0;
};
const plan = result => JSON.parse(right(compile(right(result))));

test("every public, experimental and inherited Buildx option is a typed named module argument", () => {
  for (const node of snapshot.commands) {
    const name = pure(node.path);
    const endpoint = schema.exports[command(node.path)];
    const input = schema.components.schemas[endpoint.input.$ref.split("/").at(-1)];
    const flags = new Map();
    for (let depth = 0; depth <= node.path.length; depth++) {
      const parent = snapshot.commands.find(row => JSON.stringify(row.path) === JSON.stringify(node.path.slice(0, depth)));
      for (const option of parent.options) flags.set(option.flag, option);
    }
    for (const [flag, option] of flags) {
      const field = flag === "type" ? "type_" : camel(flag);
      assert.ok(input.properties[field], `${command(node.path)}: --${flag} is missing from module schema`);
      const kind = input.properties[field].type;
      const value = kind === "boolean" ? false : kind === "array" ? ["first", "second ' 世界; $(touch injected)"] : kind === "integer" ? 3 : "literal ' 世界; $(touch injected)";
      const decoded = right(C[`decode${title(name)}Args`]({ runDir: "run", [field]: value }));
      const document = plan(C[name](decoded));
      const expected = (Array.isArray(value) ? value : [value]).map(item => `--${flag}=${item}`);
      for (const argument of expected) assert.ok(document.tasks[0].args.includes(argument), `${name}: lost ${argument}`);
      assert.equal(document.tasks[0].program, "docker");
      assert.equal(document.inherit_env, true);
      assert.equal(document.jobs, 1);
      assert.equal(document.tasks[0].timeout_ms, 900000);
      assert.equal(option.type === "boolean", kind === "boolean");
    }
  }
  assert.equal(Object.keys(schema.exports).length, snapshot.commands.length);
});

test("build preserves argv, repeatable options and context boundaries without a shell", () => {
  const decoded = right(C.decodeBuildArgs({
    project: "/tmp/project ' 世界", runDir: "/tmp/run ' 世界", docker: "/tmp/tools/docker tool",
    context: "--literal-context", builder: "builder name", platform: ["linux/amd64", "linux/arm64"],
    tag: ["test:first", "test:second"], buildArg: ["VALUE=hello ' 世界; $(touch injected)"],
    output: ["type=local,dest=out"], pull: false, experimental: true, timeoutMs: 12345,
  }));
  const document = plan(C.build(decoded));
  const task = document.tasks[0];
  assert.equal(task.cwd, "/tmp/project ' 世界");
  assert.equal(task.program, "/tmp/tools/docker tool");
  assert.equal(task.env.BUILDX_EXPERIMENTAL, "1");
  assert.equal(task.timeout_ms, 12345);
  assert.ok(task.args.includes("--build-arg=VALUE=hello ' 世界; $(touch injected)"));
  assert.deepEqual(task.args.slice(-2), ["--", "--literal-context"]);
  assert.ok(task.args.indexOf("--builder=builder name") < task.args.indexOf("build"));
  assert.equal(document.tasks.length, 1);
});

test("nested command options stay at the scope where Docker accepts them", () => {
  const value = right(C.decodeDebugBuildArgs({ runDir: "run", experimental: true, invoke: "sh", on: "error", tag: ["test:image"] }));
  const args = plan(C.debugBuild(value)).tasks[0].args;
  assert.ok(args.indexOf("debug") < args.indexOf("--invoke=sh"));
  assert.ok(args.indexOf("--invoke=sh") < args.indexOf("build"));
  assert.ok(args.indexOf("--tag=test:image") > args.indexOf("build"));
});

test("missing options preserve Docker defaults, including default-true flags", () => {
  const request = right(C.decodeImagetoolsCreateArgs({ runDir: "run" }));
  const args = plan(C.imagetoolsCreate(request)).tasks[0].args;
  assert.ok(!args.some(value => value.startsWith("--prefer-index")));
  const explicit = right(C.decodeImagetoolsCreateArgs({ runDir: "run", preferIndex: false }));
  assert.ok(plan(C.imagetoolsCreate(explicit)).tasks[0].args.includes("--prefer-index=false"));
});

test("typed decoders and plan validation reject invalid requests before execution", () => {
  for (const request of [[], { runDir: "run", platform: "linux/arm64" }, { runDir: "run", push: "true" },
    { runDir: "run", tag: [1] }, { runDir: "run", extraArgs: ["--unknown"] }, { runDir: "run", push: null }]) {
    assert.equal(C.decodeBuildArgs(request).constructor.name, "Left");
  }
  for (const request of [{}, { runDir: "" }, { runDir: "run", project: "" }, { runDir: "run", docker: "" },
    { runDir: "run", timeoutMs: 0 }, { runDir: "run", context: "bad\0path" }, { runDir: "run", tag: ["bad\0tag"] }]) {
    assert.equal(C.build(right(C.decodeBuildArgs(request))).constructor.name, "Left");
  }
});
