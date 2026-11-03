"""Real Extism/XTP ABI tests; no package providers or installers are invoked."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
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

    def test_nested_existing_plugin(self):
        command = f"{WASM / 'env.wasm'} get_env FLUENTCI_WASM_TEST"
        self.assertEqual(self.call("module", "call", command), "héllo 世界")

    def test_nested_xtp_generated_plugin_host_imports(self):
        command = f"{WASM / 'xtp_smoke.wasm'} env_roundtrip nested-unicode-世界"
        self.assertEqual(self.call("module", "call", command), "nested-unicode-世界")

    def test_invalid_json_export_input_fails(self):
        self.call("xtp_smoke", "echo_json", "not-json", success=False)

    def test_missing_export_fails(self):
        self.call("xtp_smoke", "does_not_exist", success=False)


if __name__ == "__main__":
    unittest.main()
