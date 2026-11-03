import * as F from "../src/files.ts";
import * as M from "../src/matrix.ts";
import { dag } from "../src/plugin.ts";
export function greet() {
  const words = ["/usr/bin/printf", "%s", Host.inputString()].map(word => "'" + word.replace(/'/g, "'\\''") + "'");
  Host.outputString(dag.pipeline("typescript-greeting").withExec(words).stdout());
}

export function matrix() {
  const prepare = { ...M.task("prepare", "/usr/bin/python3", ["-c", "from pathlib import Path; Path('seed.txt').write_text('seed')"]),
    cache: { inputs: ["mode.txt"], outputs: ["seed.txt"], toolchain: "matrix-prepare-v1" } };
  let cases = M.product(M.axis("node", ["22", "24"]), M.axis("suite", ["unit", "integration"]));
  cases = M.include(M.exclude(cases, ([node,suite]) => node === "22" && suite === "integration"), [M.named("experimental", ["26", "unit"] as const)]);
  const jobs = M.matrix("test", cases, ([node,suite]) => M.after([prepare], {
    ...M.task("ignored", "/usr/bin/python3", ["job.py", node, suite]),
    cache: { inputs: ["seed.txt", "mode.txt", "job.py"], outputs: [`out-${node}-${suite}`], toolchain: "matrix-v1" },
  }));
  const join = M.after(jobs, { ...M.task("join", "/usr/bin/python3", ["-c", "from pathlib import Path; Path('joined').write_text('done')"]), needs: ["prepare"] });
  const report = M.run({ ...M.parallel([prepare, ...jobs, join], 2), fail_fast: false, cache_dir: "action-cache" }, Host.inputString());
  if (report.outcome !== "succeeded") throw Error("matrix jobs failed; see the run result");
  Host.outputString("matrix complete");
}

export function workflow() {
  const task = (id: string, mode: string) => M.task(id, "/usr/bin/python3", ["job.py", mode]);
  const producer = M.resource(M.artifact(M.output(M.retry(task("produce", "produce"), 3, 10), "version", "value.txt"), "binary", "artifact.bin"), "checkout");
  const consumer = M.outputEnv(task("consume", "consume"), "VALUE", producer, "version");
  const optional = M.allowFailure(task("optional", "optional"));
  const never = M.when(task("never", "never"), "never");
  const failure = M.when(M.after([consumer], task("failure", "failure")), "failure");
  const cleanup = M.always(M.after([consumer, optional, never, failure], task("cleanup", "cleanup")));
  const report = M.run(M.parallel([producer, consumer, optional, never, failure, cleanup], 2), Host.inputString());
  if (report.outcome !== "succeeded") throw Error("workflow failed; see result.json");
  Host.outputString("workflow complete");
}

export function files() {
  const directory = F.mkdir("directory", "data", true);
  const write = M.after([directory], F.writeText("write", "data/version.env", "VERSION='héllo 世界; $(touch injected)'\n"));
  const copy = M.after([write], F.copyFile("copy", "data/version.env", "copied.env"));
  const read = M.after([copy], F.readText("read", "copied.env"));
  const hash = M.after([copy], F.hash("hash", "copied.env"));
  const env = M.after([copy], F.envFile(M.task("env", "/usr/bin/printenv", ["VERSION"]), "copied.env"));
  const report = M.run(M.parallel([directory, write, copy, read, hash, env], 2), Host.inputString());
  if (report.outcome !== "succeeded") throw Error("native files failed; see result.json");
  Host.outputString("files complete");
}
