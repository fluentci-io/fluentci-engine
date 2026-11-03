"""Real uv provisioning and Python project execution, only on disposable CI hosts."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", ROOT / "target/release/fluentci-engine")).resolve()
PLUGIN = ROOT / "target/wasm32-unknown-unknown/release/uv.wasm"


class UvProject(unittest.TestCase):
    def test_missing_uv_is_provisioned_and_project_arguments_remain_literal(self):
        with tempfile.TemporaryDirectory(prefix="fluentci-uv-") as directory:
            root = Path(directory)
            project = root / "Python project with spaces"
            shutil.copytree(ROOT / "fixtures/uv-demo", project)
            environment = dict(os.environ, PATH="/usr/bin:/bin:/usr/sbin:/sbin",
                               UV_UNMANAGED_INSTALL=str(root / "bin"), UV_PYTHON=sys.executable,
                               UV_PYTHON_DOWNLOADS="never", UV_CACHE_DIR=str(root / "cache"))
            expected = ["hello world", "quoted ' value", "$(touch unexpected)", "世界"]
            command = [str(BINARY), "call", "-m", str(PLUGIN), "run",
                       json.dumps([str(project), "main.py", *expected], ensure_ascii=False)]
            for attempt in range(2):
                result = subprocess.run(command, env=environment, capture_output=True,
                                        text=True, timeout=180)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                document = next(line for line in reversed(result.stdout.splitlines()) if line.startswith("{"))
                self.assertEqual(json.loads(document), {"message": "hello from uv", "args": expected})
                self.assertTrue((root / "bin" / "uv").is_file())
                self.assertFalse((project / "unexpected").exists())
                if attempt == 0:
                    # The second call must reuse the installed binary.
                    environment["UV_OFFLINE"] = "true"


if __name__ == "__main__":
    unittest.main()
