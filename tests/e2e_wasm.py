"""Real Extism/XTP ABI tests; no package providers or installers are invoked."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

BINARY = Path(os.environ.get("FLUENTCI_ENGINE_BIN", "target/release/fluentci-engine")).resolve()
WASM = Path(os.environ.get("FLUENTCI_WASM_DIR", "target/wasm32-unknown-unknown/release")).resolve()


class WasmContracts(unittest.TestCase):
    def call(self, plugin, function, value="", *, success=True):
        environment = dict(os.environ, FLUENTCI_WASM_TEST="héllo 世界")
        result = subprocess.run(
            [str(BINARY), "call", "-m", str(WASM / f"{plugin}.wasm"), function, value],
            capture_output=True, text=True, timeout=30, env=environment,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout.removesuffix("\n")

    def test_official_xtp_generated_json_export(self):
        document = dict(args='unicode 世界, two  spaces, newline\nand "quotes"', function="", url="local")
        output = self.call("xtp_smoke", "echo_json", json.dumps(document, ensure_ascii=False))
        self.assertEqual(json.loads(output), document)

    def test_official_xtp_host_imports_preserve_utf8_json_pairs(self):
        value = '  héllo 世界\nwith  spaces and "quotes"  '
        self.assertEqual(self.call("xtp_smoke", "env_roundtrip", value), value)

    def test_existing_pdk_text_and_json_boolean_imports(self):
        self.assertEqual(self.call("env", "get_env", "FLUENTCI_WASM_TEST"), "héllo 世界")
        self.assertIs(json.loads(self.call("env", "has_env", "FLUENTCI_WASM_TEST")), True)
        self.assertIs(json.loads(self.call("env", "has_env", "FLUENTCI_WASM_TEST_ABSENT_93B2")), False)

    def test_existing_pdk_file_handle_and_hash_imports(self):
        with tempfile.TemporaryDirectory(prefix="fluentci-wasm-") as root:
            path = Path(root) / "payload with spaces"
            payload = b"fluentci\x00wasm\n"
            path.write_bytes(payload)
            # Legacy graph progress is printed before the plugin's final return value.
            output = self.call("hash", "sha256", str(path))
            self.assertEqual(output.splitlines()[-1], hashlib.sha256(payload).hexdigest())

    def test_legacy_host_pipeline_runs_through_executor(self):
        output = self.call("env", "exec", "printf kernel-executed")
        self.assertEqual(output.splitlines()[-1], "kernel-executed")

    @unittest.skipUnless(os.name == "posix", "Unix process group cancellation")
    def test_ctrl_c_cancels_legacy_kernel_children(self):
        import signal
        import time
        with tempfile.TemporaryDirectory(prefix="fluentci-legacy-cancel-") as directory:
            marker = Path(directory) / "pid"
            # Temporary directory paths contain no shell metacharacters.
            command = f"echo $$ > '{marker}'; exec sleep 100"
            process = subprocess.Popen([str(BINARY), "call", "-m", str(WASM / "env.wasm"), "exec", command],
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                deadline = time.monotonic() + 15
                while (not marker.exists() or marker.stat().st_size == 0) and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.02)
                self.assertTrue(marker.exists(), "legacy child did not start")
                pid = int(marker.read_text())
                process.send_signal(signal.SIGINT)
                stdout, stderr = process.communicate(timeout=15)
                self.assertNotEqual(process.returncode, 0, (stdout, stderr))
                with self.assertRaises(ProcessLookupError):
                    os.kill(pid, 0)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()

    def test_legacy_services_are_supervised_without_process_compose(self):
        with tempfile.TemporaryDirectory(prefix="fluentci-legacy-service-") as root:
            output = self.call("env", "service_smoke", root)
            self.assertEqual(output.splitlines()[-1], "healthy")
            pid = int((Path(root) / "service.pid").read_text())
            import time
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                try:
                    os.kill(pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.05)
            else:
                self.fail(f"legacy service process {pid} survived pipeline completion")

    def test_nested_existing_plugin(self):
        command = f"{WASM / 'env.wasm'} get_env FLUENTCI_WASM_TEST"
        self.assertEqual(self.call("module", "call", command), "héllo 世界")

    def test_nested_xtp_generated_plugin_host_imports(self):
        command = f"{WASM / 'xtp_smoke.wasm'} env_roundtrip nested-unicode-世界"
        self.assertEqual(self.call("module", "call", command), "nested-unicode-世界")

    def test_wasm_traps_report_the_root_cause_on_stderr(self):
        with tempfile.TemporaryDirectory(prefix="fluentci-traps-") as directory:
            script = Path(__file__).resolve().parents[1] / ".github/scripts/create-bsd-smoke-fixtures.py"
            subprocess.run([sys.executable, str(script), directory], check=True, timeout=5)
            for function, cause in [("unreachable", "wasm `unreachable` instruction executed"),
                                    ("memory_oob", "out of bounds memory access")]:
                with self.subTest(function=function):
                    result = subprocess.run([str(BINARY), "call", "-m", str(Path(directory) / f"{function}.wasm"), function],
                                            capture_output=True, text=True, timeout=30)
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn("Wasm export", result.stderr)
                    self.assertIn(cause, result.stderr)

    def test_invalid_json_export_input_fails(self):
        self.call("xtp_smoke", "echo_json", "not-json", success=False)

    def test_missing_export_fails(self):
        self.call("xtp_smoke", "does_not_exist", success=False)


if __name__ == "__main__":
    unittest.main()
