"""Portable matrix fixture: bounded overlap, distinct outputs, failure and cancellation."""
import json
import pathlib
import sys
import time

node, suite = sys.argv[1:]
mode = pathlib.Path("mode.txt").read_text().strip()
started = time.monotonic_ns()
pathlib.Path(f"start-{node}-{suite}").write_text(str(started))
pathlib.Path(f"pid-{node}-{suite}").write_text(str(__import__("os").getpid()))
if mode == "sleep":
    time.sleep(120)
else:
    deadline = time.monotonic() + 10
    while len(list(pathlib.Path(".").glob("start-*"))) < 2 and time.monotonic() < deadline:
        time.sleep(0.01)
    time.sleep(0.3)
    if mode == "fail" and (node, suite) == ("22", "unit"):
        sys.exit(7)
    pathlib.Path(f"out-{node}-{suite}").write_text(json.dumps({"node": node, "suite": suite, "start": started, "end": time.monotonic_ns()}))
    print(f"done {node}/{suite}")
