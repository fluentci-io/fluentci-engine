import assert from "node:assert/strict";
import { test } from "node:test";
import * as gleam from "../sdk/gleam/src/plugin_ffi.mjs";
import * as rescript from "../sdk/rescript/src/guest_ffi.mjs";

// Model the guest memory ABI rather than an Extism host runtime.
function guest() {
  const values = new Map();
  const calls = [];
  let next = 1n;
  const allocate = value => { const offset = next++; values.set(offset, value); return offset; };
  globalThis.Memory = {
    fromString: value => ({ offset: allocate(value) }),
    fromJsonObject: value => ({ offset: allocate(JSON.stringify(value)) }),
    find: offset => ({
      readString: () => { assert(values.has(offset)); return values.get(offset); },
      readJsonObject: () => { assert(values.has(offset)); return JSON.parse(values.get(offset)); },
    }),
  };
  globalThis.Host = { getFunctions: () => new Proxy({}, {
    get: (_, name) => offset => {
      if (offset !== undefined) assert.equal(typeof offset, "bigint", "imports receive pointers, not memory objects");
      const input = offset === undefined ? undefined : values.get(offset);
      calls.push({ name, input });
      if (["pipeline", "as_service", "cache", "nix"].includes(name)) return allocate(JSON.stringify({ id: `${name}-id` }));
      if (name === "stdout") return allocate("héllo 世界");
      if (name === "stderr") throw Error("engine failure");
    },
  }) };
  return calls;
}
const ok = value => ({ ok: value });
const error = value => ({ error: value });

test("Gleam builders are pure, immutable, and preserve argv at the guest boundary", () => {
  const calls = guest();
  const args = ["%s", "spaces 'quote'; $(touch injected) 世界"];
  const command = gleam.exec_step("printf", args);
  args.push("mutated");
  assert(Object.isFrozen(command));
  assert(Object.isFrozen(command.args));
  assert.equal(calls.length, 0);
  assert.deepEqual(gleam.run("build", [command], false, ok, error), ok("héllo 世界"));
  const words = JSON.parse(calls.find(call => call.name === "with_exec").input);
  assert.equal(words.length, 3);
  assert.equal(words[2], "'spaces '\\''quote'\\''; $(touch injected) 世界'");
});

test("Gleam services lower before clients and caches use canonical JSON objects", () => {
  const calls = guest();
  const service = gleam.service_step("db", "database", [gleam.exec_step("db", [])]);
  const steps = [service, gleam.cache_step("cargo", "target"), gleam.provider_step("nix", true)];
  assert.equal(calls.length, 0);
  gleam.run("client", steps, false, ok, error);
  assert(calls.findIndex(call => call.name === "as_service") < calls.findIndex(call => call.name === "with_service"));
  assert.deepEqual(JSON.parse(calls.find(call => call.name === "with_cache").input), { id: "cache-id", key: "cargo", path: "target" });
  assert.deepEqual(JSON.parse(calls.find(call => call.name === "nix").input), { impure: true });
});

test("ReScript values are reusable and terminal errors become typed results", () => {
  const calls = guest();
  const original = rescript.pipeline("build");
  const configured = rescript.env(rescript.exec(original, ["printf", "%s", "héllo 世界"]), "MODE", "release");
  assert.equal(original.steps.length, 0);
  assert.equal(calls.length, 0);
  assert.deepEqual(rescript.run(configured, false, ok, error), ok("héllo 世界"));
  assert.deepEqual(JSON.parse(calls.find(call => call.name === "set_envs").input), [["MODE", "release"]]);
  assert.deepEqual(rescript.run(configured, true, ok, error), error("engine failure"));
});

test("NUL arguments fail before with_exec reaches the engine", () => {
  const calls = guest();
  const plan = rescript.exec(rescript.pipeline("build"), ["printf", "bad\0argument"]);
  assert.match(rescript.run(plan, false, ok, error).error, /NUL/);
  assert(!calls.some(call => call.name === "with_exec"));
});

test("TypeScript Pipeline.withExec sends the canonical JSON array to the host", async () => {
  const calls = guest();
  const { dag } = await import("../sdk/typescript/src/plugin.ts");
  const args = ["printf", "%s", "héllo 世界; 'quotes' and  two spaces"];
  const output = dag.pipeline("build").withExec(args).stdout();
  assert.equal(output, "héllo 世界");
  assert.deepEqual(JSON.parse(calls.find(call => call.name === "with_exec").input), args);
});
