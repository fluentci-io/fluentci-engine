"""One workflow contract through seven compiled Wasm guest SDKs."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from e2e_matrix import BINARY, LANGUAGES, ROOT, plugin

VALUE = "héllo 世界; $(touch injected) 'quotes' and  two spaces"

class WorkflowGuests(unittest.TestCase):
    def test_each_sdk_retries_binds_outputs_publishes_artifacts_and_finalizes(self):
        for language in LANGUAGES:
            with self.subTest(language=language), tempfile.TemporaryDirectory(prefix=f"fluentci {language} workflow ") as directory:
                root = Path(directory)
                shutil.copy2(ROOT / "fixtures/workflow-demo/job.py", root / "job.py")
                run = root / "run"
                result = subprocess.run([str(BINARY), "call", "-m", str(plugin(language)), "workflow", str(run)],
                                        cwd=root, capture_output=True, text=True, timeout=45)
                diagnostics = result.stdout + result.stderr
                if (run / "result.json").exists():
                    diagnostics += (run / "result.json").read_text()
                self.assertEqual(result.returncode, 0, diagnostics)
                report = json.loads((run / "result.json").read_text())
                self.assertEqual(report["outcome"], "succeeded", diagnostics)
                tasks = {task["id"]: task for task in report["tasks"]}
                producer = tasks["produce"]
                self.assertEqual([a["exit_code"] for a in producer["attempts"]], [7, 0])
                self.assertIn("retry-me", Path(producer["stdout"]).read_text())
                self.assertEqual(producer["outputs"]["version"], VALUE)
                artifact = producer["artifacts"]["binary"]
                self.assertEqual(Path(artifact["path"]).read_bytes(), b"verified artifact\x00\xff")
                self.assertEqual(hashlib.sha256(Path(artifact["path"]).read_bytes()).hexdigest(), artifact["sha256"])
                (root / "artifact.bin").unlink()
                self.assertTrue(Path(artifact["path"]).is_file())
                self.assertEqual(tasks["optional"]["status"], "failed")
                self.assertTrue(tasks["optional"]["allowed_failure"])
                for name in ["never", "failure"]:
                    self.assertEqual(tasks[name]["status"], "skipped")
                    self.assertTrue(tasks[name]["condition_skipped"])
                self.assertEqual(tasks["cleanup"]["status"], "succeeded")
                self.assertTrue((root / "cleaned").is_file())
                self.assertTrue((root / "consumed").is_file())
                self.assertFalse((root / "unexpected").exists())
                self.assertFalse((root / "injected").exists())

if __name__ == "__main__":
    unittest.main()
