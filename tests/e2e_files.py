"""Native file and dotenv task contracts through all compiled guest languages."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from e2e_matrix import BINARY, LANGUAGES, plugin

TEXT = "VERSION='héllo 世界; $(touch injected)'\n"
class FileGuests(unittest.TestCase):
    def test_cli_preserves_native_filesystem_diagnostics(self):
        with tempfile.TemporaryDirectory(prefix="fluentci file error ") as directory:
            root = Path(directory)
            plan = {"version":1,"inherit_env":False,"tasks":[{"id":"read","cwd":str(root),"file_operation":{"op":"read_text","path":"missing.txt"}}]}
            plan_path = root / "plan.json"
            plan_path.write_text(json.dumps(plan))
            process = subprocess.run([str(BINARY), "execute", str(plan_path), "--run-dir", str(root / "run"), "--progress", "plain"], cwd=root, capture_output=True, text=True, timeout=10)
            self.assertEqual(process.returncode,1,process.stdout+process.stderr)
            report = json.loads((root / "run/result.json").read_text())
            task = report["tasks"][0]
            self.assertIsNone(task["exit_code"])
            self.assertIn("missing.txt",task["error"])
            self.assertIn("file operation",task["error"])
            self.assertIn("missing.txt",process.stderr)

    def test_every_sdk_runs_native_files_and_literal_env_files(self):
        for language in LANGUAGES:
            with self.subTest(language=language), tempfile.TemporaryDirectory(prefix="fluentci files ") as directory:
                root = Path(directory)
                run = root / "run"
                process = subprocess.run([str(BINARY), "call", "-m", str(plugin(language)), "files", str(run)], cwd=root, capture_output=True, text=True, timeout=30)
                diagnostic = process.stdout + process.stderr
                self.assertEqual(process.returncode, 0, diagnostic)
                report = json.loads((run / "result.json").read_text())
                self.assertEqual(report["outcome"], "succeeded", report)
                tasks = {task["id"]: task for task in report["tasks"]}
                self.assertEqual(tasks["read"]["file_result"], TEXT)
                self.assertEqual(tasks["hash"]["file_result"], hashlib.sha256(TEXT.encode()).hexdigest())
                self.assertEqual((root / "copied.env").read_text(), TEXT)
                self.assertEqual(Path(tasks["env"]["stdout"]).read_text(), "héllo 世界; $(touch injected)\n")
                self.assertTrue(all(task["exit_code"] is None for name,task in tasks.items() if name != "env"))
                self.assertFalse((root / "injected").exists())
                # Loading the dotenv file itself must never log its values.
                self.assertEqual(Path(tasks["env"]["stderr"]).read_text(), "")
if __name__ == "__main__":
    unittest.main()
