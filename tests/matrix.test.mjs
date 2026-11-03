import assert from "node:assert/strict";
import { test } from "node:test";
import * as M from "../sdk/typescript/src/matrix.ts";
import * as R from "../sdk/rescript/src/matrix_ffi.mjs";

const ok = value => ({ value });
const error = error => ({ error });

test("typed products, explicit includes, predicate excludes, stable IDs and joins", () => {
  let cases = M.product(M.axis("node", ["22", "24"]), M.axis("suite", ["unit", "integration"]));
  cases = M.exclude(cases, ([node,suite]) => node === "22" && suite === "integration");
  cases = M.include(cases, [M.named("experimental", ["26", "unit"])]);
  const tasks = M.matrix("test", cases, value => M.task("ignored", "/missing-program", value, "/missing-directory"));
  assert.deepEqual(tasks.map(task => task.id), ["test[node=22,suite=unit]", "test[node=24,suite=unit]", "test[node=24,suite=integration]", "test[experimental]"]);
  assert.equal(M.parallel(tasks, 2).jobs, 2);
  const join = M.after(tasks, { ...M.task("join", "test", []), needs: ["prepare"] });
  assert.deepEqual(join.needs, ["prepare", ...tasks.map(task => task.id)]);
  const reverse = M.matrix("test", [...cases].reverse(), value => M.task("ignored", "test", value));
  assert.deepEqual(reverse.map(task => task.id).reverse(), tasks.map(task => task.id));
  assert.equal(M.named("a,b=世界", null).key, "a%2Cb%3D%E4%B8%96%E7%95%8C");
  assert(Object.isFrozen(tasks));
});

test("ReScript transport mirrors matrix identity and preserves typed tuple values", () => {
  const cases = R.product(R.axis("node", ["22", "24"]), R.axis("suite", ["unit", "integration"]), ok, error).value;
  const tasks = R.matrix("test", cases, value => R.task("ignored", "test", value, "."), ok, error).value;
  assert.equal(tasks[0].id, "test[node=22,suite=unit]");
  assert.deepEqual(tasks[0].args, ["22", "unit"]);
  assert.equal(R.parallel(tasks, 3).jobs, 3);
});

test("invalid expansions are bounded and reject duplicates before builder effects", () => {
  let called = false;
  const build = () => { called = true; return M.task("bad", "bad", []); };
  assert.throws(() => M.matrix("test", [M.named("same", 1), M.named("same", 2)], build), /unique/);
  assert.equal(called, false);
  assert.throws(() => M.product(M.axis("a", Array(65).fill("x")), M.axis("b", Array(65).fill("y"))), /4096/);
  assert.throws(() => M.compile(M.parallel([], 0)), /positive/);
});
