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

## Overlapping input snapshot workload

```sh
cargo test --release -p fluentci-executor --lib overlapping_input_snapshot_benchmark -- --ignored --nocapture
```

This opt-in benchmark snapshots one 8 MiB file through 32 alternating declarations
of its containing directory and the file itself, checking exact snapshot equality
against the single-root case. Five samples are printed; no timing threshold is
used. In one local release comparison, the median dropped from 478.906 ms to
15.144 ms after skipping paths already visited within the same snapshot (about
32× for this deliberately overlapping workload). This does not imply a similar
speedup for ordinary pipelines with disjoint declarations.

The snapshot map is local to each traversal. Input hashing before execution and
before cache publication still reads file contents independently, as do workspace
copy verification snapshots. There is no cross-task or metadata-only digest cache.
Regression coverage checks both parent-first and child-first declarations,
unchanged snapshot digests, subsequent content mutations, missing paths and
cancellation. Re-run on the same idle host to compare; OS page caches are not flushed.

## Inactive-task journal polling workload

```sh
cargo test --release -p fluentci-executor --lib inactive_task_polling_benchmark -- --ignored --nocapture
```

This benchmark creates 2,000 task records: 1,000 pending tasks without log files
and 1,000 completed tasks with empty stdout/stderr files. After the initial journal
update, each of five samples measures 20 ordinary polling ticks. It isolates
journal polling rather than process execution. Skipping unopened pending logs and
terminal streams already observed at EOF reduced a local release median from
355.425 ms to 0.810 ms. These are synthetic polling timings, not whole-pipeline
speedup claims. State comparison still walks the task list each tick.

Live tasks continue to be polled. Completed streams with unread backlogs continue
until EOF; the run's final drain rechecks every terminal stream before emitting
`run_finished`. Regression coverage exercises all four stdout/stderr/probe streams,
output larger than the per-tick budget, later task starts, appended live output,
final EOF rechecks, and duplicate-free journal capture. Raw log files remain the
source of exact bytes; journal text remains a lossy UTF-8 display representation.


## Duplicate-content cache publication workload

```sh
cargo test --release -p fluentci-executor --lib duplicate_blob_publication_benchmark -- --ignored --nocapture
```

This benchmark publishes 16 separate 4 MiB output files plus stdout/stderr with
identical contents. After one initial publication populates the local CAS, five
samples time another publication, including source snapshots, executable/input
verification, blob verification, and synced manifest replacement. It then restores
all outputs and logs and checks their exact bytes. OS page caches are not flushed;
there is no timing threshold.

On one local Darwin ARM64 release comparison, the median fell from 273.872 ms to
149.451 ms (about 45% less time) after verifying each unique blob once per
publication. This deliberately duplicate-heavy workload does not represent a
general pipeline speedup. Distinct source output paths are still fully hashed;
the optimization avoids repeated verification of the same CAS destination.

The verified-digest set is discarded after each save. Regression coverage corrupts
a shared blob between saves, verifies repair, changes one output and one log, and
checks that restoration retains every file's correct contents. Verification on
restore and the staged-publication protocol are unchanged.

## Copied workspace materialization

```sh
cargo test --release -p fluentci-executor --lib workspace_copy_benchmark -- --ignored --nocapture
cargo build --release -p fluentci-engine
python3 benchmarks/workspace.py --samples 5 --mib 64
```

The first command compares a forced streaming copy with the native-clone/fallback
path for a 64 MiB file. Each sample verifies exact contents outside the timed copy.
The second benchmark includes CLI startup, workspace materialization, all source
and destination content/mode verification, and a task that mutates its private
copy. It checks that the source is unchanged outside the timing window and emits
JSON samples. Neither benchmark flushes OS caches or imposes a timing threshold.
Use `--binary` to compare separately built release artifacts on the same host.

On one macOS 15.6 ARM64/APFS comparison, the copy-only medians were 57.358 ms for
streaming and 0.142 ms for successful native cloning. The complete CLI workload
had a 419.752 ms median with the streaming baseline (`159e4e7`) and 398.678 ms with
native cloning, about 5% less time in this pair of runs. Complete-workload samples
in milliseconds were:

- Streaming: 478.304, 462.681, 415.097, 419.752, 416.183.
- Cloning: 971.627, 398.678, 400.421, 397.170, 396.080.

The first cloned-workload sample was substantially slower; these small, sequential
sample sets are observations, not a general throughput guarantee. Content hashing
remains in both paths, and small files, unsupported filesystems, cross-device
copies, or macOS metadata fallback can erase the copy-only advantage. Cloning
avoids eagerly copying data blocks; it does not eliminate verification reads.

Local tests require actual native cloning with
`FLUENTCI_REQUIRE_WORKSPACE_CLONE=1 cargo test --release -p fluentci-executor workspace_copy`.
They also exercise forced streaming, independent mutations, permission preservation,
existing-target rejection, cancellation, special-file rejection, and macOS metadata
fallback. The workflow requires cloning on its macOS jobs; Linux native behavior
still requires a reflink-capable filesystem. Linux release cross-check and Clippy
validation are compile checks, not native runtime evidence.

The platform calls follow [Apple's clonefile contract](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/clonefile.2)
and [Linux's FICLONE contract](https://www.man7.org/linux/man-pages/man2/FICLONE.2const.html).
