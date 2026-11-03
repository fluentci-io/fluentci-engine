"""Generate tiny Wasm trap fixtures without invoking any compiler."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
root.mkdir(parents=True, exist_ok=True)


def section(kind, data):
    assert len(data) < 128
    return bytes([kind, len(data)]) + data


for name, memory, code in [
    ("unreachable", False, b"\x00"),
    # Load from the first byte beyond the current linear memory, computed at runtime.
    ("memory_oob", True, b"\x3f\x00\x41\x10\x74\x28\x02\x00"),
]:
    module = b"\x00asm\x01\x00\x00\x00"
    module += section(1, b"\x01\x60\x00\x01\x7f")
    module += section(3, b"\x01\x00")
    if memory:
        module += section(5, b"\x01\x00\x01")
    export = name.encode("ascii")
    module += section(7, bytes([1, len(export)]) + export + b"\x00\x00")
    body = b"\x00" + code + b"\x0b"
    module += section(10, bytes([1, len(body)]) + body)
    (root / f"{name}.wasm").write_bytes(module)
