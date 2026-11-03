import * as M from "../src/matrix.ts";
import { dag } from "../src/plugin.ts";
export function greet() {
  const words = ["/usr/bin/printf", "%s", Host.inputString()].map(word => "'" + word.replace(/'/g, "'\\''") + "'");
  Host.outputString(dag.pipeline("typescript-greeting").withExec(words).stdout());
}

export function matrix() {
  const prepare = M.task("prepare", "/usr/bin/python3", ["-c", "from pathlib import Path; Path('seed.txt').write_text('seed')"]);
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
