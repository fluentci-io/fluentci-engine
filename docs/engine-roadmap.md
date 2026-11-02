# Host-native engine implementation ledger

The product goal is a Dagger alternative for non-containerized pipelines, with
functional SDKs and excellent local/server CI/CD DX. Work remains active; these
milestones do not establish a claim of being the fastest engine.

## Implemented in this worktree

- Explicit versioned plans with dependency validation, bounded parallel tasks,
  task-local environments, timeouts, process-group cancellation, resource names,
  disk-backed logs, and structured results.
- Legacy graph dependency/order/output regression fixes; legacy APIs remain
  sequential pending migration to the explicit plan runtime.
- Opt-in local content-addressed action cache, verified artifact restoration,
  invalidation components, and Unix action-lock deduplication.
- Durable JSON execution events and unified live CLI TUI, plain CI output, and
  machine-readable event output.
- Rust extension adapters for Homebrew (macOS ARM64/Linux x86_64), MacPorts
  (macOS; native or macports-rs), FreeBSD pkg, NetBSD pkgin, OpenBSD pkg_add.
- Local fixture, executor, cache, CLI e2e, and PTY tests. GitHub native-host/VM
  workflows are configured but not remotely executed/verified yet.

## Required follow-through

1. Homebrew, MacPorts/native-or-rust, and BSD providers now work in explicit local
   and server plans and the PureScript SDK. Installation is an uncached DAG step;
   shared consumers and exclusive installers coordinate per prefix. Complete other
   SDK/legacy entry points, custom-prefix/shell-hook handling, and verified package
   state snapshots for toolchain caching.
2. The sibling PureScript SDK now provides pure immutable host plans, typed remote
   execution, and a direct Node adapter with Aff cancellation. Its end-to-end tests
   exercise this release engine. Implement equivalent TypeScript, Gleam, and Elixir
   builders/adapters, automate cross-repository compatibility tests, and complete
   the legacy SDK migration path.
3. Independent authenticated server runs now support submission, status, cancellation,
   persisted results, resumable events/raw logs, restart reconciliation, and remote
   trace context. Follow through with SDK migration, idempotent submission, durable
   admission queues, history indexing, and run-store retention. The legacy GraphQL
   server remains separate.
4. Cross-run resource leases now coordinate declared resource names across local
   and server processes. Add isolated workspaces, cross-run deadlock/fairness policy,
   independent crash supervision, and explicit declared I/O semantics beyond the
   current user-supplied cache contract.
5. Extend cache support with contained symlinks, remote storage, cache inspection
   and explanation commands, retention/GC, and scalable hashing/transfer scheduling.
6. Validate the GitHub matrix on actual remote runners. Add native provisioning
   and integration jobs for MacPorts and macports-rs. Homebrew and BSD native
   package-plan installation tests are configured but await remote verification.
   BSD kernel/package-manager tests do not prove Wasmtime/legacy server portability.
7. Benchmark orchestration, cold/warm cache, large logs/artifacts, cancellation,
   and independent task scaling. Optimize based on measured bottlenecks.
8. Improve UX for local/server workflows: concise actionable errors, attach to
   running jobs, inspect task logs/results, explain cache decisions, and sensible
   zero-daemon local defaults.

Current boundaries and commands are documented in [host-executor.md](host-executor.md).
Do not mark the overall goal complete based only on the tests for one milestone.

## Build policy

Always build, run, test, and benchmark optimized release artifacts. Use
`--release` for Cargo build/run/test/check/clippy commands; CLI e2e tests default
to `target/release/fluentci-engine`. Do not report debug-profile timings.

## Additional requested work

- First-class run-scoped services are implemented: readiness gates, bounded startup
  and recurring health probes, consumer failure propagation, TERM/KILL shutdown,
  reverse dependency teardown, and cancellation/crash tests. Follow through with
  cross-run endpoint allocation and server/SDK integration.
- Per-step OpenTelemetry is implemented for host tasks with correlated journal
  events and exporter shutdown. Service lifecycle events are implemented. Extend this to metrics,
  remote propagation, and server execution. HTTP/protobuf export and failed-run
  flushing are covered by a local receiver e2e test.
- Keep changes in multiple conventional commits, including the Node 26 mise update.
