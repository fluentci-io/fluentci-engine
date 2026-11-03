from __future__ import annotations

import extism
from fluentci import pipeline


@extism.plugin_fn
def greet() -> None:
    name = extism.input_str()
    output = pipeline("python-greeting").exec("/usr/bin/printf", "%s", name).stdout()
    extism.output_str(output)


@extism.plugin_fn
def matrix() -> None:
    from fluentci import matrix as M
    from fluentci.plan import Task, Plan, Cache
    prepare = Task("prepare", "/usr/bin/python3", ("-c", "from pathlib import Path; Path('seed.txt').write_text('seed')"))
    cases = M.product(M.axis("node", ["22", "24"]), M.axis("suite", ["unit", "integration"]))
    cases = M.include(M.exclude(cases, lambda pair: pair == ("22", "integration")), [M.named("experimental", ("26", "unit"))])
    def build(pair: tuple[str, str]) -> Task:
        node, suite = pair
        return M.after([prepare], Task("ignored", "/usr/bin/python3", ("job.py", node, suite),
            cache=Cache(("seed.txt", "mode.txt", "job.py"), (f"out-{node}-{suite}",), "matrix-v1")))
    jobs = M.matrix("test", cases, build)
    join = M.after(jobs, Task("join", "/usr/bin/python3", ("-c", "from pathlib import Path; Path('joined').write_text('done')"), needs=("prepare",)))
    plan = Plan((prepare, *jobs, join), jobs=2, fail_fast=False)
    plan.run(extism.input_str())
    extism.output_str("matrix complete")
