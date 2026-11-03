#!/usr/bin/env python3
"""Exercise release cache snapshots using temporary data, without compilation."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("snapshot-rust-cache.sh").resolve()


class RustCacheSnapshot(unittest.TestCase):
    def test_snapshot_copies_cache_contents_and_preserves_empty_directories(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            cache = root / "mounted cache"
            cargo = root / "cargo home"
            output = root / "unmounted snapshot"
            cache.mkdir()
            (cargo / "registry" / "empty").mkdir(parents=True)
            (cargo / "git").mkdir()
            (cache / ".entry").write_text("cached object")
            (cargo / "git" / "checkout").write_text("git object")
            environment = dict(os.environ, SCCACHE_DIR=str(cache), CARGO_HOME=str(cargo))
            subprocess.run(["sh", str(SCRIPT), str(output)], env=environment, check=True)
            self.assertEqual((output / "sccache" / ".entry").read_text(), "cached object")
            self.assertEqual((output / "git" / "checkout").read_text(), "git object")
            self.assertTrue((output / "registry" / "empty").is_dir())
            (cache / ".entry").write_text("changed after snapshot")
            self.assertEqual((output / "sccache" / ".entry").read_text(), "cached object")

    def test_missing_cache_source_is_reported(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            environment = dict(os.environ, SCCACHE_DIR=str(root / "missing"))
            result = subprocess.run(["sh", str(SCRIPT), str(root / "snapshot")],
                                    env=environment, capture_output=True)
            self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
