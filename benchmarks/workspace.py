"""Release CLI workspace benchmark, including content verification and task startup."""
import argparse
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/fluentci-engine"))
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--mib", type=int, default=64)
    parser.add_argument("--files", type=int, default=1, help="number of files, each --mib MiB")
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()
    if min(args.samples, args.mib, args.files, args.jobs) < 1:
        parser.error("samples, mib, files and jobs must be positive")
    binary = args.binary.resolve(strict=True)
    env = {k: v for k, v in os.environ.items() if not k.startswith("OTEL_")}
    with tempfile.TemporaryDirectory(prefix="fluentci-workspace-bench-") as temporary:
        root = Path(temporary)
        source = root / "source"
        source.mkdir()
        content = b"x" * (args.mib * 1024 * 1024)
        for index in range(args.files):
            (source / f"input-{index}").write_bytes(content)
        plan = root / "plan.json"
        plan.write_text(json.dumps(dict(
            version=1, inherit_env=False, jobs=args.jobs,
            workspace=dict(source=str(source), inputs=[f"input-{index}" for index in range(args.files)]),
            tasks=[dict(id="probe", program="/bin/sh", args=["-c", "printf changed > input-0"], cwd=".")],
        )))
        samples = []
        for index in range(args.samples):
            run = root / f"run-{index}"
            started = time.perf_counter()
            result = subprocess.run(
                [str(binary), "execute", str(plan), "--run-dir", str(run), "--progress", "plain"],
                env=env, capture_output=True, timeout=120,
            )
            elapsed = time.perf_counter() - started
            if result.returncode:
                raise RuntimeError(result.stderr.decode(errors="replace"))
            report = json.loads((run / "result.json").read_text())
            assert report["tasks"][0]["status"] == "succeeded", report
            for file in range(args.files):
                assert (run / f"workspace/input-{file}").read_bytes() == (b"changed" if file == 0 else content)
                assert (source / f"input-{file}").read_bytes() == content
            samples.append(elapsed)
        print(json.dumps(dict(
            platform=platform.platform(), binary=str(binary), mib_per_file=args.mib, files=args.files, jobs=args.jobs,
            samples_seconds=samples, median_seconds=statistics.median(samples),
            scope="CLI startup, workspace copy and verification, one mutating task; excludes fixture creation and post-run assertions",
        ), indent=2))


if __name__ == "__main__":
    main()
