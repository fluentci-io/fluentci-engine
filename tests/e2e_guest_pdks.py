"""Build artifacts from six SDKs must invoke the actual FluentCI PDK through Wasm."""
import os
import hashlib
import json
import shutil
import signal
import time
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", ROOT / "target/release/fluentci-engine")).resolve()


class GuestPDKs(unittest.TestCase):
    def test_each_language_calls_native_processes_through_engine_pdk(self):
        for language in ["python", "elixir", "gleam", "rescript", "purescript", "typescript"]:
            with self.subTest(language=language), tempfile.TemporaryDirectory(prefix="fluentci guest ") as directory:
                artifact = ROOT / "sdk" / language
                if language != "python":
                    artifact /= "demo-plugin"
                artifact /= "dist/plugin.wasm"
                self.assertTrue(artifact.is_file(), f"missing compiled {language} plugin: {artifact}")
                value = "héllo 世界; $(touch injected) 'quotes' and  two spaces"
                result = subprocess.run(
                    [str(BINARY), "call", "-m", str(artifact), "greet", value],
                    cwd=directory, capture_output=True, text=True, timeout=90,
                )
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                # Captured command output and CLI return-value printing each add
                # a newline. Ignore only line terminators, preserving spaces.
                self.assertEqual(result.stdout.rstrip("\r\n").splitlines()[-1], value, result.stdout)
                self.assertFalse((Path(directory) / "injected").exists(), "exec interpreted a literal argument as shell code")



class PureScriptGuestContracts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="fluentci PS guest ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.plugin = self.root / "plugin.wasm"
        shutil.copy2(ROOT / "sdk/purescript/demo-plugin/dist/plugin.wasm", self.plugin)

    def call(self, function, value="", *, success=True):
        result = subprocess.run([str(BINARY), "call", "-m", str(self.plugin), function, value],
                                cwd=self.root, capture_output=True, text=True, timeout=45)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def assert_reaped(self, name):
        pid = int((self.root / name).read_text())
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                return
            time.sleep(0.05)
        self.fail(f"supervised process {pid} survived guest completion/cancellation")

    def test_failures_reach_the_plugin_error_boundary(self):
        result = self.call("failure", success=False)
        self.assertIn("exit status 7", result.stderr)
        invalid = self.call("invalid", success=False)
        self.assertIn("Invalid argument (exec)", invalid.stderr)

    def test_file_hash_and_copy_preserve_paths_with_spaces_and_quotes(self):
        path = self.root / "source 'quotes' 世界.txt"
        path.write_text("file contents 世界")
        result = self.call("hash_file", str(path))
        self.assertEqual(result.stdout.rstrip("\r\n").splitlines()[-1], hashlib.sha256(path.read_bytes()).hexdigest())
        self.call("copy_file", str(path))
        self.assertEqual((self.root / "copied file.txt").read_bytes(), path.read_bytes())

    def test_nested_calls_preserve_input(self):
        value = "nested 世界; 'quotes' and  spaces"
        result = self.call("nested", value)
        self.assertEqual(result.stdout.rstrip("\r\n"), value)

    def test_secret_injection_does_not_print_the_value(self):
        secret = "fixture-secret-世界-123"
        result = self.call("secret_env", secret)
        self.assertEqual(result.stdout.rstrip("\r\n").splitlines()[-1], "present")
        self.assertNotIn(secret, result.stdout + result.stderr)

    def test_services_are_ready_and_reaped_after_guest_completion(self):
        result = self.call("service")
        self.assertEqual(result.stdout.rstrip("\r\n").splitlines()[-1], "healthy")
        self.assert_reaped("service.pid")

    def test_cancellation_reaps_guest_tasks_and_services(self):
        proc = subprocess.Popen([str(BINARY), "call", "-m", str(self.plugin), "cancel"],
                                cwd=self.root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 20
            while not (self.root / "client.ready").exists() and proc.poll() is None and time.monotonic() < deadline:
                time.sleep(0.05)
            self.assertTrue((self.root / "client.ready").exists(), "guest client never started")
            proc.send_signal(signal.SIGINT)
            stdout, stderr = proc.communicate(timeout=15)
            self.assertNotEqual(proc.returncode, 0, stdout + stderr)
            self.assert_reaped("client.pid")
            self.assert_reaped("service.pid")
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.communicate(timeout=5)

    def test_guest_generated_host_plan_restores_and_invalidates_action_cache(self):
        (self.root / "seed.txt").write_text("first")
        plan = json.loads(self.call("action_plan").stdout)
        plan["cache_dir"] = str(self.root / "action-cache")
        manifest = self.root / "plan.json"
        manifest.write_text(json.dumps(plan))
        for index, expected in enumerate(["stored", "hit", "stored"]):
            if index == 1:
                (self.root / "output.txt").unlink()
            if index == 2:
                (self.root / "seed.txt").write_text("changed")
            run = self.root / f"run-{index}"
            result = subprocess.run([str(BINARY), "execute", str(manifest), "--run-dir", str(run), "--progress", "plain"],
                                    cwd=self.root, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            report = json.loads((run / "result.json").read_text())
            self.assertEqual(report["tasks"][0]["cache"]["outcome"], expected)
            self.assertEqual((self.root / "output.txt").read_text(), "changed" if index == 2 else "first")
            self.assertEqual((self.root / "executions").read_text(), "runrun" if index == 2 else "run")


if __name__ == "__main__":
    unittest.main()
