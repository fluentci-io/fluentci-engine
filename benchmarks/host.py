"""Repeatable release-CLI benchmark; timings include process startup and reporting."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import tempfile
import time


def measure(binary, root, name, tasks, jobs):
    plan = root / f"{name}.json"
    run = root / f"run-{name}"
    plan.write_text(json.dumps(dict(version=1, inherit_env=False, jobs=jobs,
                                    cache_dir=str(root / "cache"), tasks=tasks)))
    started = time.perf_counter()
    process = subprocess.run([str(binary), "execute", str(plan), "--run-dir", str(run),
                              "--progress", "plain"], stdout=subprocess.DEVNULL,
                             stderr=subprocess.PIPE, timeout=120)
    elapsed = time.perf_counter() - started
    if process.returncode:
        raise RuntimeError(process.stderr.decode(errors="replace"))
    report = json.loads((run / "result.json").read_text())
    if any(task["status"] != "succeeded" for task in report["tasks"]):
        raise RuntimeError(f"Unsuccessful benchmark: {report}")
    outcomes = {}
    for task in report["tasks"]:
        outcome = task["cache"]["outcome"]
        outcomes[outcome] = outcomes.get(outcome, 0) + 1
    return dict(seconds=elapsed, cache=outcomes,
                stdout_bytes=sum(Path(task["stdout"]).stat().st_size for task in report["tasks"]),
                journal_bytes=(run / "events.jsonl").stat().st_size)


def task(root, name, script, needs=()):
    return dict(id=name, cwd=str(root), program=sys.executable,
                args=["-c", script], needs=list(needs), timeout_ms=30000)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/fluentci-engine"))
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--tasks", type=int, default=32)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--artifact-mib", type=int, default=16)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if min(args.samples, args.tasks, args.jobs, args.artifact_mib) < 1:
        parser.error("all sizes/counts must be positive")
    binary = args.binary.resolve(strict=True)
    # Prevent accidentally presenting debug timings as the release baseline.
    if "debug" in binary.parts:
        parser.error("use a release binary")
    data = dict(schema_version=1, platform=platform.platform(), machine=platform.machine(),
                cpu_count=os.cpu_count(), python=sys.version, binary=str(binary),
                binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                settings=dict(samples=args.samples, tasks=args.tasks, jobs=args.jobs,
                              artifact_mib=args.artifact_mib), scenarios={})
    with tempfile.TemporaryDirectory(prefix="fluentci-bench-") as directory:
        for sample in range(args.samples):
            root = Path(directory) / str(sample)
            root.mkdir()
            scenarios = [("empty", [], args.jobs)]
            independent = [task(root, f"task-{i}", "pass") for i in range(args.tasks)]
            chain = [task(root, f"task-{i}", "pass", [f"task-{i-1}"] if i else [])
                     for i in range(args.tasks)]
            sleepers = [task(root, f"task-{i}", "import time; time.sleep(0.05)")
                        for i in range(args.tasks)]
            scenarios += [("independent", independent, args.jobs), ("chain", chain, args.jobs),
                          ("sleep_serial", sleepers, 1), ("sleep_parallel", sleepers, args.jobs)]
            scenarios.append(("logs_1mib", [task(root, "logs",
                "import sys; sys.stdout.buffer.write(b'x' * 1048576)")], 1))
            cached = task(root, "artifact", "from pathlib import Path; "
                          f"Path('artifact').write_bytes(b'x' * {args.artifact_mib * 1048576})")
            cached["cache"] = dict(inputs=[], outputs=["artifact"], toolchain=sys.version)
            scenarios += [("cache_cold", [cached], 1), ("cache_warm", [cached], 1)]
            for name, tasks, jobs in scenarios:
                if name == "cache_warm":
                    (root / "artifact").unlink()  # Measure actual restoration, not existing outputs.
                result = measure(binary, root, name, tasks, jobs)
                if name == "logs_1mib" and result["stdout_bytes"] != 1048576:
                    raise RuntimeError("Incomplete log capture")
                if name.startswith("cache_"):
                    expected = "stored" if name == "cache_cold" else "hit"
                    if result["cache"] != {expected: 1}:
                        raise RuntimeError(f"Unexpected cache outcome: {result}")
                    artifact = root / "artifact"
                    if artifact.stat().st_size != args.artifact_mib * 1048576:
                        raise RuntimeError("Restored artifact size mismatch")
                data["scenarios"].setdefault(name, {"samples": []})["samples"].append(result)
                print(f"sample {sample+1}/{args.samples} {name}: {result['seconds']:.4f}s", file=sys.stderr)
    for result in data["scenarios"].values():
        seconds = sorted(sample["seconds"] for sample in result["samples"])
        result.update(median_seconds=statistics.median(seconds),
                      p95_seconds=seconds[math.ceil(0.95 * len(seconds)) - 1],
                      min_seconds=seconds[0], max_seconds=seconds[-1])
    data["sleep_speedup"] = (data["scenarios"]["sleep_serial"]["median_seconds"] /
                             data["scenarios"]["sleep_parallel"]["median_seconds"])
    args.output.write_text(json.dumps(data, indent=2) + "\n")


if __name__ == "__main__":
    main()
