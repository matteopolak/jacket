"""Consume generated ICO and private group bytes using real LLVM SDK tools."""
from pathlib import Path
import shutil
import struct
import subprocess
import sys

# tools/, for folder-qualified imports
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from toolkit.host import brew_prefix

root = Path(sys.argv[1]).resolve()
brew = brew_prefix()
rc = shutil.which("llvm-rc") or str(brew / "opt/llvm/bin/llvm-rc" if brew else "llvm-rc")
linker = shutil.which("lld-link") or str(brew / "bin/lld-link" if brew else "lld-link")
icon_path = root / "resource.ico"
rc_path = root / "resource.rc"
res_path = root / "resource.res"
rc_path.write_text(f'101 ICON "{icon_path.as_posix()}"\n')
subprocess.run([rc, "/fo", res_path.name, rc_path.name], cwd=root, check=True)
data = res_path.read_bytes()
entries = {}
cursor = 0

def identifier(offset):
    value, = struct.unpack_from("<H", data, offset)
    if value == 0xffff:
        value, = struct.unpack_from("<H", data, offset + 2)
        return value, offset + 4
    start = offset
    while struct.unpack_from("<H", data, offset)[0]:
        offset += 2
    return data[start:offset].decode("utf-16-le"), offset + 2

while cursor < len(data):
    size, header_size = struct.unpack_from("<II", data, cursor)
    resource_type, position = identifier(cursor + 8)
    name, position = identifier(position)
    position = (position + 3) & ~3
    _, _, language, _, _ = struct.unpack_from("<IHHII", data, position)
    assert header_size >= position + 16 - cursor
    payload = data[cursor + header_size:cursor + header_size + size]
    assert len(payload) == size
    if size:
        entries[resource_type, name, language] = payload
    cursor += header_size + ((size + 3) & ~3)
assert cursor == len(data) and len(entries) == 6
icon = icon_path.read_bytes()
for index in range(5):
    size, offset = struct.unpack_from("<II", icon, 14 + 16 * index)
    assert entries[3, 1 + index, 1033] == icon[offset:offset + size]
assert entries[14, 101, 1033] == (root / "group.bin").read_bytes()
dll = root / "resource-only.dll"
subprocess.run([linker, "/dll", "/noentry", "/machine:x64",
                f"/out:{dll.name}", res_path.name], cwd=root, check=True)
assert dll.read_bytes()[:2] == b"MZ"
print("LLVM RC group/image bytes match; SDK resource-only PE linked")
