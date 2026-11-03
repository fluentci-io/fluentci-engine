# Host-native engine implementation ledger

The product goal is a Dagger alternative for non-containerized pipelines, with
functional SDKs and excellent local/server CI/CD DX. Work remains active; these
milestones do not establish a claim of being the fastest engine.

## Implemented in this worktree

- Opt-in per-run copies of declared source paths, with content/mode verification,
  cancellable setup, CLI/HTTP support, and a PureScript workspace builder. This
  separates run files but does not sandbox host access. Explicit relocatable cache
  contracts enable reuse/deduplication across copied workspaces; the caller must
  guarantee path-independent outputs and logs.

- Explicit versioned plans with dependency validation, bounded parallel tasks,
  task-local environments, timeouts, process-group cancellation, resource names,
  disk-backed logs, and structured results.
- Legacy graph dependency/order/output regression fixes; legacy APIs remain
  sequential pending migration to the explicit plan runtime.
- Opt-in local content-addressed action cache, verified artifact restoration,
  invalidation components, Unix action-lock deduplication, rollback on restoration
  errors, preflight rejection of unordered overlapping declared cache paths, and
  bounded background cache preparation/restoration/saving that retains leases
  and dependency gates.
- Durable JSON execution events and unified live CLI TUI, plain CI output, and
  machine-readable event output.
- Rust extension adapters for Homebrew (macOS ARM64/Linux x86_64), MacPorts
  (macOS; native or macports-rs), FreeBSD pkg, NetBSD pkgin, OpenBSD pkg_add.
- Local fixture, executor, cache, CLI e2e, and PTY tests. GitHub native-host/VM
  workflows, including native/Rust MacPorts on macOS ARM64 and x86_64, are
  configured but not remotely executed/verified yet.

## Required follow-through

1. Homebrew, MacPorts/native-or-rust, and BSD providers now work in explicit local
   and server plans and the PureScript SDK. Installation is an uncached DAG step;
   shared consumers and exclusive installers coordinate per prefix. Complete other
   SDK/legacy entry points, custom-prefix/shell-hook handling, and verified package
   state snapshots for toolchain caching.
2. The sibling PureScript SDK now provides pure immutable host plans, typed remote
   execution, and a direct Node adapter with Aff cancellation. Its end-to-end tests
   exercise this release engine. The Gleam JavaScript SDK now has pure pipe-friendly
   host builders, typed direct local execution with cooperative cancellation, and
   typed HTTP submission/state/results/cancellation with acknowledged keyed replay
   and resumable binary task/service-probe logs plus typed execution journal events.
   Typed cursor-paginated history is verified across concurrent admissions and server restart.
   Explicit submission trace context is verified through journal correlation and a local
   OTLP collector, including original-trace preservation on keyed replay.
   Release-engine cache/workspace, process-cleanup and server fixtures run on Node and Deno.
   The host CI matrix now includes these fixtures on both runtimes with required
   release-engine execution (configured, not remotely verified). Implement
   equivalent TypeScript and Elixir builders/adapters, automate cross-repository
   compatibility tests, and complete
   the legacy SDK migration path.
3. Independent authenticated server runs now support submission, status, cancellation,
   persisted results, authenticated cursor-paginated history, resumable events/raw logs, restart reconciliation, and remote
   trace context and durable keyed submission replay. Follow through with SDK migration, durable
   admission queues, an on-disk history index, and run-store retention. History now
   uses an ordered in-memory index rebuilt at startup and updated on admission. The legacy GraphQL
   server remains separate.
4. Cross-run resource leases now coordinate declared resource names across local
   and server processes. Add isolated workspaces, cross-run deadlock/fairness policy,
   independent crash supervision, and explicit declared I/O semantics beyond the
   current user-supplied cache contract.
5. Saved-run `cache-diff` now compares task key components without exposing values.
   Extend cache support with contained symlinks, remote storage, live cache inspection,
   retention/GC, and scalable hashing/transfer scheduling. Overlapping input declarations
   now reuse entries within a single snapshot, with unchanged snapshot digests and
   independent content reads across preparation/publication checks.
6. Validate the GitHub matrix on actual remote runners. Add native provisioning
   and integration jobs for MacPorts and macports-rs. Homebrew and BSD native
   package-plan installation tests are configured but await remote verification.
   BSD kernel/package-manager tests do not prove Wasmtime/legacy server portability.
7. Shared-path validation now reuses per-task graph reachability and avoids retaining
   all ordered pairs; a 1,000-writer regression and four-node DAG model checks cover
   scale and ordering correctness. The release CLI benchmark in `benchmarks/host.py` now measures empty runs,
   dependency chains, independent tasks, bounded parallelism, 1 MiB logs, and
   cold/warm artifact caching with outcome checks and machine-readable samples.
   Extend measurements to cancellation, large DAGs, services, server throughput,
   and competing engines; optimize based on measured bottlenecks. Journal polling now
   skips pending-task files and terminal streams already at EOF on ordinary ticks,
   while preserving active streams, bounded backlog reads and a final EOF recheck.
   An opt-in release microbenchmark measures inactive-task polling separately.
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
  and recurring health probes (optionally distinct commands and time limits), consumer failure propagation, TERM/KILL shutdown,
  reverse dependency teardown, separate raw probe diagnostic logs with resumable
  HTTP/SDK access and live journal/TUI display, and cancellation/crash tests. Follow through with
  cross-run endpoint allocation and server/SDK integration.
- Per-step OpenTelemetry is implemented for host tasks with correlated journal
  events and exporter shutdown. Service lifecycle events are implemented. Extend this to metrics,
  remote propagation, and server execution. HTTP/protobuf export and failed-run
  flushing are covered by a local receiver e2e test.
- Keep changes in multiple conventional commits, including the Node 26 mise update.
