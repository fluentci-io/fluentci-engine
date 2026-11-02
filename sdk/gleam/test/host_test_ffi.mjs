import { mkdtempSync, writeFileSync, readFileSync, existsSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";

export function inspect(encoded) {
  const p = JSON.parse(encoded), [install, task] = p.tasks;
  return p.version === 1 && p.inherit_env === false && p.workspace.source === "/source" &&
    JSON.stringify(p.workspace.inputs) === '["src"]' &&
    install.package_install.provider.manager === "macports" &&
    install.package_install.provider.backend === "rust" &&
    install.package_install.provider.executable === "/tools/port" &&
    JSON.stringify(install.package_install.packages) === '["jq"]' &&
    task.env.__proto__ === "preserved" && task.env.VALUE === "new" &&
    JSON.stringify(task.args) === JSON.stringify(["literal; $VALUE", "two words", 'quote"\n']) &&
    JSON.stringify(task.needs) === '["packages"]' &&
    task.package_environment[0].backend === "rust" &&
    task.service.health.timeout_ms === 100 && task.service.readiness.interval_ms === 10 &&
    !Object.hasOwn(task, "cache") && !Object.hasOwn(p, "resource_dir");
}

export function execute_fixture(build) {
  const executable = process.env.FLUENTCI_ENGINE_BIN;
  if (!executable) {
    console.error("Gleam release-engine integration skipped: set FLUENTCI_ENGINE_BIN");
    return true;
  }
  const binary = resolve(executable);
  if (binary.split(/[\\/]/).includes("debug")) throw Error("use a release engine binary");
  const root = mkdtempSync(join(tmpdir(), "fluentci-gleam-"));
  try {
    const plan = join(root, "plan.json");
    writeFileSync(plan, build(root, join(root, "cache")));
    for (const [run, expected] of [["cold", "stored"], ["warm", "hit"]]) {
      const directory = join(root, run);
      const result = spawnSync(binary, ["execute", plan, "--run-dir", directory, "--progress", "json"], { encoding: "utf8", timeout: 15000 });
      if (result.error || result.status !== 0) throw Error(result.stderr || String(result.error));
      const report = JSON.parse(readFileSync(join(directory, "result.json"), "utf8"));
      if (report.outcome !== "succeeded" || report.tasks.some(task => task.cache.outcome !== expected)) throw Error(JSON.stringify(report));
      if (readFileSync(join(directory, "workspace/consumed"), "utf8") !== "literal; $VALUE") throw Error("artifact mismatch");
    }
    if (existsSync(join(root, "artifact")) || existsSync(join(root, "consumed"))) throw Error("source checkout modified");
    return true;
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
