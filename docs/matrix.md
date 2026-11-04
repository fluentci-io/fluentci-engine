# Parallel matrix jobs

Available in FluentCI Engine **0.6.1** and matching SDKs through `execute_plan`.
Engine 0.6.0 does not contain this API.

A matrix is a pure expansion into ordinary host tasks. The existing executor owns
scheduling, dependencies, caching, logs, services, timeouts and cancellation.
Guest plugins submit the whole plan through one `execute_plan` PDK call; matrix
members never concurrently mutate the legacy current-pipeline graph.

## Compose cases

- `axis(name, values)` names string values, preserving their order.
- `named(name, value)` associates a stable label with an arbitrary typed value.
- `product(left, right)` constructs typed pairs in left-major Cartesian order.
- `exclude(cases, predicate)` removes matching values.
- `include(cases, extra)` appends explicitly named cases.
- `matrix(group, cases, build)` assigns scoped IDs to tasks returned by a pure builder.
- `after(parents, task)` adds dependencies while retaining existing ones.
- `parallel(tasks, jobs)` creates a plan with an explicit concurrency bound.

Argument order follows each language's composition convention: PureScript takes
configuration first and the receiver last for `#`; Gleam, Elixir and ReScript use
value-first pipes. Product/matrix validation uses each language's normal
`Result`/`Either`, exception, or tagged-result conventions.

Case IDs look like `test[node=22,suite=unit]`. Labels use UTF-8 RFC3986 escaping,
so commas, equals signs and Unicode remain unambiguous. Filtering/reordering does
not renumber IDs. Matrix group names accept 1..64 ASCII letters, digits, dots,
underscores and hyphens. Duplicate/empty keys are rejected before invoking the
builder. Products and matrix expansions are bounded to 4096 cases.

Case labels appear in logs and reports. Generic named cases can carry typed
configuration without automatically putting its value in the task ID.

## Python guest

```python
import extism
from fluentci.matrix import axis, product, exclude, include, named, matrix, parallel
from fluentci.plan import Task

@extism.plugin_fn
def tests() -> None:
    cases = product(axis("node", ["22", "24"]), axis("suite", ["unit", "integration"]))
    cases = exclude(cases, lambda pair: pair == ("22", "integration"))
    cases = include(cases, [named("experimental", ("26", "unit"))])
    jobs = matrix("test", cases, lambda pair:
        Task("assigned-by-matrix", "/usr/bin/printf", ("%s/%s\n", *pair)))
    report = parallel(jobs, jobs=2).run(extism.input_str())
    if not report.succeeded:
        raise RuntimeError("matrix failed; inspect the run result")
    extism.output_str("matrix complete")
```

The axes above are configuration values: they do not automatically install or
select Node versions. A real test task must select its runtime explicitly through
its program/arguments, environment, or package setup dependencies.

## PureScript guest

```purescript
import Data.Tuple (Tuple(..))
import FluentCI.Matrix as M
import FluentCI.Host.Plan as H

jobs = do
  cases <- M.product (M.axis "node" ["22", "24"])
                     (M.axis "suite" ["unit", "integration"])
  let selected = cases
        # M.exclude (\pair -> pair == Tuple "22" "integration")
        # M.include [M.named "experimental" (Tuple "26" "unit")]
  M.matrix "test" selected \(Tuple node suite) ->
    H.task "assigned-by-matrix" "/usr/bin/printf" "."
      # H.withArgs ["%s/%s\n", node, suite]

-- Handle the Either, then execute through FluentCI.Plugin:
-- P.runPlan "fresh-run-directory" (M.parallel 2 tasks)
```

`P.runPlan` returns a typed terminal report. Inspect its outcome/task statuses to
choose whether your plugin should fail. A failed job report is distinct from an
invalid request or host-call failure.

## Rust/native plans

`fluentci_executor::matrix` and `fluentci_executor::primitives` expose the same
case/product/filter/expansion functions. `Case<T>` preserves typed values and
`matrix` overwrites the builder's placeholder task ID with the scoped case ID.
`parallel` returns an ordinary `Plan`, so it works through `execute`, `serve-runs`,
and native library execution without a second scheduler or plan protocol.

## Other SDKs

| SDK | Case/plan API | Guest effect |
| --- | --- | --- |
| TypeScript | `sdk/typescript/src/matrix.ts` | `run(plan, runDir)` |
| ReScript | `Matrix` | `Matrix.run(plan, runDir)` |
| Gleam | `fluentci/matrix`, `fluentci/host` | `matrix.run(plan, runDir)` |
| Elixir | `FluentCI.Matrix`, `FluentCI.Host` | DSL `run_plan(plan, input())` |

The Elixir DSL expands cases at build time and compiles the native plan into its
Wasm guest. `run_plan` fails the export when a job fails by default; use
`fail_on_error: false` to return the report JSON instead. This remains the
supported CI DSL subset, not general Elixir/BEAM execution inside Wasm.

## Prerequisites, joins, and failures

Add common setup tasks as dependencies in each case builder. Add every expanded
member to a final task's dependencies with `after` (`after_jobs`/`afterJobs` in
SDKs where that is the idiomatic name). This retains the final task's existing
prerequisites. Joins require successful parents; use the returned report for
aggregation that must also handle failed cases. Joining an empty group adds no
new dependencies.

`jobs` limits the entire plan. There is no separate per-matrix concurrency quota
in this version. `fail_fast` is also plan-wide: false lets independent cases
finish after another fails; true cancels remaining work. The native dependency
rules skip consumers of failed tasks in either mode.

## Cache, workspace, and lifecycle contracts

Cache reusable setup tasks as well as their matrix consumers. An uncached
prerequisite carries a per-run dependency identity, intentionally invalidating
its cached descendants even when their directly declared input files match.
A stable cached prerequisite allows verified warm hits without weakening the
engine's dependency contract.

Each expanded case has its own task status, journal events, stdout/stderr paths,
cache policy and telemetry. Pass parameters into argv/environment/toolchain
inputs so they affect action-cache keys; labels alone are not a cache contract.
Declare separate outputs per case or explicit ordering/resources. A matrix does
not automatically create independent working directories or package prefixes.

Relative task/cache/resource paths are resolved against the engine invocation's
working directory. Workspace source paths resolve there too, while task paths in
a copied workspace remain workspace-relative. `execute plan.json` instead uses
the plan file's directory as its base. Cached tasks must choose an explicit
`cache_dir`; the examples use a shared per-workspace `action-cache` directory.

The guest effect takes UTF-8 JSON `{ "plan": ..., "run_dir": "fresh-path" }` and
returns terminal `RunResult` JSON. The caller owns a fresh run directory and keeps
its `result.json`, `events.jsonl`, and logs. Paths/plan validation happen before
execution. Task failures return a report; malformed requests/preflight errors may
fail the host call. Cancellation shares the owning engine run's token, reaps its
process groups, and persists the cancelled result. Fatal Wasm/host traps may end
an invocation before the guest can return a typed result.

All members execute on the current host. OS labels do not provision remote
workers, VMs, containers, or other operating systems. Legacy guest `Pipeline`
values retain serial ordering; build native tasks for a parallel matrix.

## Run the end-to-end example

Every `sdk/*/demo-plugin` exports `matrix`; Python uses `sdk/python/demo/plugin.py`.
Build the desired guest using its SDK guide, then from this checkout:

```sh
repo="$PWD"
cd fixtures/matrix-demo
"$repo/target/release/fluentci-engine" call \
  -m "$repo/sdk/purescript/demo-plugin/dist/plugin.wasm" matrix "$PWD/run-1"
```

The fixture runs four parameterized jobs with two slots, a prerequisite and a
success-only join. Engine CI checks actual overlap, cache hits/restoration,
failure propagation, and cancellation through compiled Wasm for all six languages.
