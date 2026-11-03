"""Run the checked-in PureScript host example against the release engine."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", ROOT / "target/release/fluentci-engine")).resolve()
example = ROOT / "sdk/purescript/demo-host"
encoded = subprocess.check_output(["node", "--input-type=module", "-e",
    'import { main } from "./output/Main/index.js"; main()'], cwd=example)
plan = json.loads(encoded)
assert plan["tasks"][0]["cache"]["toolchain_files"] == ["/bin/cat"]
with tempfile.TemporaryDirectory(prefix="fluentci-purescript-example-") as temp:
    root = Path(temp)
    (root / "plan.json").write_bytes(encoded)
    (root / "input").write_text("first")
    env = {key: value for key, value in os.environ.items() if not key.startswith("OTEL_")}
    for index, expected in enumerate(["stored", "hit", "stored"]):
        if index == 2:
            (root / "input").write_text("changed")
        subprocess.run([str(BINARY), "execute", str(root / "plan.json"), "--run-dir",
                        str(root / f"run-{index}"), "--progress", "plain"], check=True, env=env, timeout=15)
        report = json.loads((root / f"run-{index}/result.json").read_text())
        assert report["tasks"][0]["cache"]["outcome"] == expected, report
        assert "toolchain_files" in report["tasks"][0]["cache"]["components"]
        assert (root / "output").read_text() == ("changed" if index == 2 else "first")
print("PureScript example: cache store, hit, and input invalidation passed")
