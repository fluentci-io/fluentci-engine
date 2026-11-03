import assert from "node:assert/strict";
import { test } from "node:test";
import * as T from "../sdk/typescript/src/matrix.ts";
import * as R from "../sdk/rescript/src/matrix_ffi.mjs";

for (const [name, sdk] of [["TypeScript", T], ["ReScript", R]]) {
  test(`${name}: workflow composition is immutable and creates real dependency bindings`, () => {
    const original = sdk.task("produce", "/missing", ["literal; $(touch unexpected)"], "/missing");
    const produced = sdk.resource(sdk.artifact(sdk.output(sdk.retry(original, 3, 10), "version", "version.txt"), "report", "report.txt"), "checkout");
    assert.equal(original.policy, undefined);
    assert.equal(produced.policy.retry.max_attempts, 3);
    const consumer = sdk.outputEnv(sdk.task("consume", "/missing", [], "/missing"), "VERSION", produced, "version");
    assert.deepEqual(consumer.needs, ["produce"]);
    assert.deepEqual(consumer.policy.bindings.VERSION, { task: "produce", name: "version" });
    const finalizer = sdk.always(consumer);
    assert.equal(finalizer.policy.when, "always");
    assert.equal(finalizer.timeout_ms, 30000);
    assert.equal(finalizer.resource_wait_timeout_ms, 30000);
    const optional = sdk.allowFailure(produced, true);
    assert.equal(optional.policy.allow_failure, true);
    assert.deepEqual(optional.policy.artifacts, { report: "report.txt" });
    assert(Object.isFrozen(produced));
  });
}
test("native service builder preserves a readiness probe without shell interpolation", () => {
  const original = T.task("database", "/missing-db", []);
  const service = T.service(original, { program: "/missing-probe", args: ["two words"], interval_ms: 50, timeout_ms: 500 });
  assert.equal(service.service.readiness.args[0], "two words");
  assert.equal(original.service, undefined);
});
