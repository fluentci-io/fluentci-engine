# Composable workflow primitives (unreleased)

These functions build immutable task values. They perform no process, filesystem,
network or provisioning operations during construction. All execution uses the
same versioned native plan and `execute_plan` guest boundary. Use matching source
builds of the engine and SDKs; published 0.6.0 binaries do not support task policy.

PureScript example:

```purescript
let
  build = Plan.task "build" "/usr/bin/python3" "."
    # Plan.withArgs ["build.py"]
    # Plan.retry 3 250
    # Plan.output "version" "version.txt"
    # Plan.artifact "binary" "dist/app.tar.gz"
    # Plan.resource "checkout:app"
  publish = Plan.task "publish" "/tools/publish" "."
    # Plan.outputEnv "VERSION" build "version"
  cleanup = Plan.task "cleanup" "/bin/rm" "."
    # Plan.withArgs ["-f", "temporary-file"]
    # Plan.after [publish]
    # Plan.always
  pipeline = Plan.plan [build, publish, cleanup] # Plan.withJobs 2
-- P.runPlan "fresh-run-directory" pipeline is the Wasm execution boundary.
```

## Functions and contracts

| Primitive | Behavior |
| --- | --- |
| `retry` | Opt-in retry of nonzero process exits. The attempt limit includes the first execution; bounded exponential backoff releases job slots and resource leases. The default cap is 60 seconds, with at most 100 attempts. Spawn failures, publication failures, timeouts and cancellation are not retried. The task timeout is a total budget across attempts/backoff. Retries can repeat side effects: use them only for suitable commands. |
| `when` | `success` (default), `failure`, `always`, or `never`, evaluated against declared dependencies. A root `failure` task has no failed dependencies and is skipped. No string-expression evaluator or serialized closures are needed. |
| `always` | A dependency-ordered finalizer that runs after success, failure or cancellation once the plan has started. It bypasses fail-fast but remains bounded by task/resource-wait timeouts (30 seconds by default). Add `after`/`needs` explicitly: an `always` task without dependencies may start immediately. A failed original task still makes the run fail. |
| `allowFailure` | Preserve the actual failed/timed-out result while allowing dependencies and the run to succeed. Attempts and errors remain inspectable. It does not turn cancellation into success. |
| `output` | Read a named UTF-8 file after successful task completion, up to 64 KiB, preserving its contents exactly. Values appear in `result.json`; use nonsecret data. |
| `outputEnv` | Bind a producer's named output to a consumer environment variable, adding the dependency automatically. The value is passed literally, never through shell interpolation, and participates in the consumer's cache key. |
| `artifact` | Copy a named regular file into private run storage, preserving its mode and verifying content before/after copying. Results expose its path, byte size and SHA-256. Artifacts survive later workspace mutation. Archive directories explicitly before publishing; links and special files are rejected. |
| `resource` | Add an exclusive cross-run resource lease without discarding previous leases. Existing shared-resource builders remain available. |
| `service` | Compose the existing foreground service lifecycle/readiness configuration with a task. Services keep independent capacity limits, health probes and supervised shutdown. |

Finalizers cannot be action cached. Services cannot use retries, conditional
execution, tolerated failure or declared products; consumers can use normal
workflow policies. Deliberately escaping process groups remains outside native
supervision guarantees. OS crashes and hard kills cannot guarantee finalizers.

Output and artifact files are collected on owned publication workers before
consumers become runnable; publication retains the task's job slot and leases.
Only successful attempts publish products. A missing/invalid declared product
fails publication. Action-cached tasks must include every product path in their
cache outputs; warm hits restore those files and republish products for the new
run. Artifact publication is local retention, independent of remote action caches.

Attempt history is retained in `tasks[].attempts`; stdout/stderr append across
attempts so earlier diagnostics remain available. Retry waits appear in the
execution journal through normal task-state/wait-reason updates. Run outcomes
account for intentionally skipped and tolerated-failure tasks while retaining
actual per-task statuses.

## SDK spelling

| SDK | Public builders |
| --- | --- |
| Rust `primitives` | `retry`, `when`, `always`, `allow_failure`, `artifact`, `output`, `output_env`, `resource`, `service` |
| PureScript `FluentCI.Host.Plan` | `retry`, `when`, `always`, `allowFailure`, `artifact`, `output`, `outputEnv`, `resource`, `service` + `asService` |
| Python `fluentci.workflow` | `retry`, `when`, `always`, `allow_failure`, `artifact`, `output`, `output_env`, `resource`, `service` |
| TypeScript `matrix` | `retry`, `when`, `always`, `allowFailure`, `artifact`, `output`, `outputEnv`, `resource`, `service` |
| ReScript `Matrix` | `retry`, `whenTask`, `always`, `allowFailure`, `artifact`, `output`, `outputEnv`, `resource`, `service` |
| Gleam `fluentci/host` | `retry`, `when_task`, `always`, `allow_failure`, `artifact`, `output`, `output_env`, `resource`, `service` + `as_service` |
| Elixir `FluentCI.Host` | `retry`, `when_task`, `always`, `allow_failure`, `artifact`, `output`, `output_env`, `resource`, `service` |

PureScript uses configuration-first, task-last arguments for `#`; other SDKs use
task-first arguments for their language's pipe/call conventions. Rust explicit
`Retry` policies support exit-code filtering and custom backoff caps.

## Retained artifacts over HTTP

Authenticated run servers provide:

```
GET /v1/runs/{run-id}/artifacts/{task-index}/{name}?cursor=0
```

Responses contain up to 4 MiB of raw bytes, `X-Next-Cursor`, `X-Artifact-Size` and
`X-Artifact-Sha256`. Continue until the cursor reaches the size, then verify the
complete download against SHA-256. Publication metadata must be present in the
terminal result. Missing names, links and unsafe paths are rejected. This API
uses the same bearer-token authorization as logs/results.

Local file paths in result metadata refer to the engine host. Remote callers
must use the authenticated download endpoint rather than interpreting those
paths as files on their own machine.
