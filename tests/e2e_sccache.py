"""Real compiler-cache provisioning and reuse. Run only on disposable CI hosts."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", ROOT / "target/release/fluentci-engine")).resolve()
PLUGIN = ROOT / "examples/sccache/dist/plugin.wasm"


def counters(stats, name):
    return sum(stats[name]["counts"].values())


class CompilerCache(unittest.TestCase):
    def test_provisions_reuses_cache_and_reaps_server(self):
        with tempfile.TemporaryDirectory(prefix="fluentci-sccache-") as directory:
            root = Path(directory)
            project = root / "Rust project ' with spaces"
            shutil.copytree(ROOT / "fixtures/sccache-demo", project)
            # Keep Rust available while excluding the action-installed sccache from PATH.
            tools = root / "tools"
            tools.mkdir()
            for name in ("cargo", "rustc", "rustdoc", "rustup"):
                executable = shutil.which(name)
                self.assertIsNotNone(executable, name)
                (tools / name).symlink_to(executable)
            environment = dict(os.environ, PATH=f"{tools}:/usr/bin:/bin:/usr/sbin:/sbin",
                               SCCACHE_INSTALL_DIR=str(root / "bin"),
                               SCCACHE_DIR=str(root / "cache"), SCCACHE_GHA_ENABLED="false")
            for attempt in range(2):
                run = root / f"run-{attempt}"
                target = root / f"target-{attempt}"
                environment["CARGO_TARGET_DIR"] = str(target)
                result = subprocess.run(
                    [str(BINARY), "call", "-m", str(PLUGIN), "build",
                     json.dumps([str(project), str(run)])], env=environment,
                    capture_output=True, text=True, timeout=240)
                diagnostics = result.stdout + result.stderr
                if (run / "result.json").exists():
                    diagnostics += (run / "result.json").read_text()
                    for log in run.glob("*.stderr"):
                        diagnostics += f"\n{log.name}:\n" + log.read_text()
                self.assertEqual(result.returncode, 0, diagnostics)
                report = json.loads((run / "result.json").read_text())
                self.assertEqual(report["outcome"], "succeeded", diagnostics)
                tasks = {task["id"]: task for task in report["tasks"]}
                stats = json.loads(Path(tasks["stats"]["stdout"]).read_text())
                self.assertGreater(counters(stats, "cache_hits" if attempt else "cache_misses"),
                                   0, json.dumps(stats, indent=2))
                self.assertTrue((target / "release/libsccache_demo.rlib").is_file())
                self.assertTrue((root / "bin/sccache").is_file())
                socket = Path("/tmp") / ("fluentci-sccache-" + hashlib.sha256(
                    str(run).encode()).hexdigest()[:24] + ".sock")
                self.assertFalse(socket.exists(), "server socket leaked")
                self.assertEqual(tasks["compiler-cache"]["status"], "succeeded", diagnostics)
                if attempt == 0:
                    # Existing binaries must be reused; deny a second installer download.
                    environment["https_proxy"] = "http://127.0.0.1:1"
                    environment["HTTPS_PROXY"] = "http://127.0.0.1:1"

            failed_run = root / "failed-run"
            failed = subprocess.run(
                [str(BINARY), "call", "-m", str(PLUGIN), "build",
                 json.dumps([str(project), str(failed_run), "--package", "missing-package"])],
                env=environment, capture_output=True, text=True, timeout=120)
            self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)
            failed_report = json.loads((failed_run / "result.json").read_text())
            self.assertEqual(failed_report["outcome"], "failed")
            self.assertTrue(any(t["id"] == "build" and t["status"] == "failed"
                                for t in failed_report["tasks"]))
            failed_socket = Path("/tmp") / ("fluentci-sccache-" + hashlib.sha256(
                str(failed_run).encode()).hexdigest()[:24] + ".sock")
            self.assertFalse(failed_socket.exists(), "failed build leaked its socket")


if __name__ == "__main__":
    unittest.main()
