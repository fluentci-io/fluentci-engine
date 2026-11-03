"""Provider subprocess contract tests. No package managers or installers run."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", "target/release/fluentci-engine")).resolve()


class ProviderWorker(unittest.TestCase):
    def invoke(self, root, commands, **options):
        request = root / "request.json"
        request.write_text(json.dumps({"version": 1, "commands": commands}))
        request.chmod(0o600)
        return subprocess.run([str(BINARY), "provider-worker", str(request)],
                              capture_output=True, text=True, timeout=20, **options)

    def command(self, root, provider, command, **extra):
        return {"provider": {"provider": provider}, "command": command,
                "cwd": str(root), **extra}

    def test_streams_are_drained_before_capture_and_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            capture = root / "captured"
            command = self.command(root, "runner", "printf 'stdout-tail'; printf 'stderr-tail' >&2",
                                   capture=str(capture), stderr=True)
            result = self.invoke(root, [command])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("stdout-tail", result.stdout)
            self.assertIn("stderr-tail", result.stdout)
            self.assertEqual(capture.read_text(), "stderr-tail\n")

    def test_hash_provider_uses_task_working_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = b"binary\x00payload\n"
            (root / "file with spaces").write_bytes(payload)
            capture = root / "captured"
            result = self.invoke(root, [self.command(root, "sha256", "file with spaces", capture=str(capture))])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(capture.read_text(), hashlib.sha256(payload).hexdigest())

    def test_service_marker_is_written_after_command_spawn(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            marker = root / "started"
            command = self.command(root, "runner", "printf service-started", started=str(marker))
            result = self.invoke(root, [command])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(marker.read_text(), "started")

    def test_failure_stops_service_chain(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            commands = [self.command(root, "runner", "exit 7"),
                        self.command(root, "runner", "touch unexpected")]
            result = self.invoke(root, commands)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((root / "unexpected").exists())

    def test_request_version_and_unknown_fields_fail_before_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for request in [{"version": 99, "commands": []},
                            {"version": 1, "commands": [], "unknown": True}]:
                path = root / "request.json"
                path.write_text(json.dumps(request))
                result = subprocess.run([str(BINARY), "provider-worker", str(path)],
                                        capture_output=True, timeout=10)
                self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
