import os
from pathlib import Path
import sys

VALUE = "héllo 世界; $(touch injected) 'quotes' and  two spaces"
mode = sys.argv[1]
if mode == "produce":
    if not Path("attempted").exists():
        Path("attempted").write_text("first")
        print("retry-me", flush=True)
        raise SystemExit(7)
    Path("value.txt").write_text(VALUE)
    Path("artifact.bin").write_bytes(b"verified artifact\x00\xff")
elif mode == "consume":
    assert os.environ["VALUE"] == VALUE
    Path("consumed").write_text("literal")
elif mode == "optional":
    raise SystemExit(4)
elif mode == "cleanup":
    Path("cleaned").write_text("done")
else:
    Path("unexpected").write_text(mode)
    raise SystemExit(9)
