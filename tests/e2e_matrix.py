"""Execute the same matrix through seven actual Wasm PDKs; no tool provisioning."""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", ROOT / "target/release/fluentci-engine")).resolve()
LANGUAGES = os.environ.get("FLUENTCI_TEST_LANGUAGES", "python,typescript,rescript,gleam,elixir,purescript,clojurescript").split(",")


def plugin(language):
    base = ROOT / "sdk" / language
    if language != "python":
        base /= "demo-plugin"
    return base / "dist/plugin.wasm"


class MatrixGuests(unittest.TestCase):
    def invoke(self, language, root, run, *, mode="ok"):
        (root / "mode.txt").write_text(mode)
        result = subprocess.run([str(BINARY), "call", "-m", str(plugin(language)), "matrix", str(root / run)],
                                cwd=root, capture_output=True, text=True, timeout=60)
        self.assertNotEqual(result.returncode, 101, result.stdout + result.stderr)
        if mode == "fail":
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return json.loads((root / run / "result.json").read_text())

    def test_each_guest_expands_schedules_joins_and_caches_its_matrix(self):
        expected = {"test[node=22,suite=unit]", "test[node=24,suite=unit]", "test[node=24,suite=integration]", "test[experimental]"}
        for language in LANGUAGES:
            with self.subTest(language=language), tempfile.TemporaryDirectory(prefix=f"fluentci {language} matrix ") as directory:
                root = Path(directory)
                for source in (ROOT / "fixtures/matrix-demo").iterdir():
                    shutil.copy2(source, root / source.name)
                report = self.invoke(language, root, "first")
                self.assertEqual(report["outcome"], "succeeded")
                rows = [task for task in report["tasks"] if task["id"].startswith("test[")]
                self.assertEqual({task["id"] for task in rows}, expected)
                self.assertEqual(len(report["tasks"]), 6)
                self.assertTrue(all(task["cache"]["outcome"] == "stored" for task in rows))
                self.assertTrue((root / "joined").is_file())
                intervals = [json.loads(path.read_text()) for path in root.glob("out-*")]
                points = sorted([(item["start"], 1) for item in intervals] + [(item["end"], -1) for item in intervals])
                active = maximum = 0
                for _, change in points:
                    active += change
                    maximum = max(maximum, active)
                self.assertEqual(maximum, 2, "matrix did not honor two concurrent job slots")
                for path in root.glob("out-*"):
                    path.unlink()
                warm = self.invoke(language, root, "warm")
                self.assertEqual(warm["outcome"], "succeeded")
                self.assertTrue(all(task["cache"]["outcome"] == "hit" for task in warm["tasks"] if task["id"] in expected), json.dumps(warm, indent=2))
                self.assertEqual(len(list(root.glob("out-*"))), 4, "cached case outputs were not restored")
                (root / "joined").unlink()
                failed = self.invoke(language, root, "failed", mode="fail")
                self.assertFalse((root / "joined").exists(), "failed matrix executed its success-only join")
                self.assertEqual(failed["outcome"], "failed")
                statuses = {task["id"]: task["status"] for task in failed["tasks"]}
                self.assertEqual(statuses["test[node=22,suite=unit]"], "failed")
                self.assertEqual(statuses["join"], "skipped")
                self.assertTrue(all(statuses[key] == "succeeded" for key in expected - {"test[node=22,suite=unit]"}), "fail_fast=false did not allow independent cases to finish")

    def test_python_host_preflight_error_does_not_panic_or_start_jobs(self):
        with tempfile.TemporaryDirectory(prefix="fluentci invalid matrix ") as directory:
            root = Path(directory)
            for source in (ROOT / "fixtures/matrix-demo").iterdir():
                shutil.copy2(source, root / source.name)
            existing = root / "already-exists"
            existing.mkdir()
            result = subprocess.run([str(BINARY), "call", "-m", str(plugin("python")), "matrix", str(existing)],
                                    cwd=root, capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertNotEqual(result.returncode, 101, result.stdout + result.stderr)
            self.assertNotIn("Cannot start a runtime", result.stderr)
            self.assertFalse(list(root.glob("pid-*")), "invalid run directory started matrix jobs")

    def test_cancelled_matrix_reaps_all_running_case_processes(self):
        for language in LANGUAGES:
            with tempfile.TemporaryDirectory(prefix="fluentci cancel matrix ") as directory:
                root = Path(directory)
                for source in (ROOT / "fixtures/matrix-demo").iterdir():
                    shutil.copy2(source, root / source.name)
                (root / "mode.txt").write_text("sleep")
                proc = subprocess.Popen([str(BINARY), "call", "-m", str(plugin(language)), "matrix", str(root / "cancelled")],
                                        cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                try:
                    deadline = time.monotonic() + 20
                    while len(list(root.glob("pid-*"))) < 2 and proc.poll() is None and time.monotonic() < deadline:
                        time.sleep(0.02)
                    self.assertEqual(len(list(root.glob("pid-*"))), 2, "matrix cases never started")
                    # A pid file can become visible between open() and write().
                    while any(not path.read_text().strip().isdigit() for path in root.glob("pid-*")) and time.monotonic() < deadline:
                        time.sleep(0.01)
                    proc.send_signal(signal.SIGINT)
                    proc.communicate(timeout=15)
                    self.assertNotEqual(proc.returncode, 0)
                    report = json.loads((root / "cancelled/result.json").read_text())
                    self.assertEqual(report["outcome"], "cancelled")
                    for path in root.glob("pid-*"):
                        pid = int(path.read_text())
                        deadline = time.monotonic() + 5
                        while time.monotonic() < deadline:
                            try:
                                os.kill(pid, 0)
                            except ProcessLookupError:
                                break
                            time.sleep(0.02)
                        else:
                            self.fail(f"cancelled matrix process {pid} survived cleanup")
                finally:
                    if proc.poll() is None:
                        proc.kill()
                        proc.communicate(timeout=5)

if __name__ == "__main__":
    unittest.main()
