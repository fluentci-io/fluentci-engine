# Host executor performance baseline

Build and measure the optimized CLI from the repository root:

```sh
cargo build --release -p fluentci-engine
python3 benchmarks/host.py --samples 5 --tasks 32 --jobs 8 --artifact-mib 16 --output /tmp/fluentci-host-benchmark.json
```

The standard-library Python harness creates temporary plans and checks successful
results, cache outcomes, restored artifact size, and exact captured log size. It
records every wall-clock sample, median, nearest-rank p95, platform, CPU count,
Python version, configuration, and executable SHA-256. Run files are removed on
completion; the JSON report remains at the requested output path. Custom release
binary paths are supported with `--binary`; paths containing `debug` are rejected.

Scenarios cover empty-run overhead, independent tasks, a serial dependency chain,
serial versus parallel 50 ms sleeps, a 1 MiB log stream, cold/warm artifact
caching, and cold/warm relocatable caching in distinct copied run workspaces. Each sample has its own workspace and action cache. The warm case deletes
the output before execution to require restoration. The copied-workspace cases
start with fresh output directories, copy a declared input of the configured
artifact size, and verify exact restored bytes and source separation. Their timed
region includes source copying and snapshot verification. "Cold" means an empty action
cache, not a flushed operating-system page cache. Python startup is part of each
task's workload; this is an end-to-end CLI measurement, not a Rust scheduler
microbenchmark. Timings include CLI startup, validation, logging, reporting, and
shutdown. Benchmark setup, executable hashing, and report parsing are excluded.

Run comparisons on the same idle machine, release toolchain, filesystem, Python
runtime, and exporter configuration. Exporters should be disabled for the default
baseline. The synthetic sleep speedup measures bounded concurrency, not a claim
about real build speed or superiority to another engine. With five samples, p95
is the maximum observation; use more samples to study tails. No timing thresholds
are enforced in CI because host load varies. Large-DAG validation, cancellation
latency, server throughput, service workloads, and competing engines still need
separate measurements.

## Shared-output validation workload

```sh
cargo test --release -p fluentci-executor --test cache large_ordered_workspace -- --nocapture
```

This regression builds 1,000 ordered tasks declaring the same output path, times
validation only, and verifies rejection after removing the last writer's dependency.
It launches no task processes and enforces no timing threshold. In one local release
comparison, validation dropped from 8.65 s to 39.8 ms after replacing per-pair graph
walks and the ordered-pair set with reusable per-task ancestor/descendant marks.
This is a targeted workload, not a general pipeline speedup claim. Overlap candidate
enumeration is still quadratic for a shared path with many writers; scratch graph
reachability storage no longer grows with the number of compared pairs. Run on the
same machine/toolchain to compare changes; filesystem validation remains included.
