"""Build artifacts from six SDKs must invoke the actual FluentCI PDK through Wasm."""
import os
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
                self.assertEqual(result.stdout.splitlines()[-1], value, result.stdout)
                self.assertFalse((Path(directory) / "injected").exists(), "exec interpreted a literal argument as shell code")


if __name__ == "__main__":
    unittest.main()
