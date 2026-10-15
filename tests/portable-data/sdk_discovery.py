"""Native Windows registry/filesystem oracle; never claims success on another OS."""
import csv
import os
from pathlib import Path
import re
import sys

if os.name != "nt":
    raise SystemExit("Native Windows is required; this is not a skipped pass")
import winreg

output = Path(sys.argv[1])
with (output / "windows-discovery.csv").open(newline="") as stream:
    row, = list(csv.reader(stream))
assert len(row) == 6
root10, root81, selected, version, library, linker = row

def query(value):
    for view in (winreg.KEY_WOW64_32KEY, winreg.KEY_WOW64_64KEY):
        try:
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,
                    r"SOFTWARE\Microsoft\Windows Kits\Installed Roots",
                    0, winreg.KEY_READ | view) as key:
                result, kind = winreg.QueryValueEx(key, value)
                if kind == winreg.REG_SZ:
                    return result
        except OSError:
            pass
    return ""

assert root10 == query("KitsRoot10") and root81 == query("KitsRoot81")
expected = None
expected_version = "0"
for root, pattern, tag in ((root10, r"(\d+)\.(\d+)\.(\d+)\.(\d+)", "10"),
                           (root81, r"winv(\d+)\.(\d+)", "8")):
    if not root or not (Path(root) / "Lib").is_dir():
        continue
    candidates = []
    for candidate in (Path(root) / "Lib").iterdir():
        match = re.fullmatch(pattern, candidate.name)
        if candidate.is_dir() and match:
            candidates.append((tuple(map(int, match.groups())), candidate))
    if candidates:
        expected = max(candidates)[1]
        expected_version = tag
        break
if expected is None:
    assert selected == "" and version == "0"
else:
    assert Path(selected).samefile(expected) and version == expected_version
if library or linker:
    assert library and linker
    assert (Path(library) / "vcruntime.lib").is_file()
    assert (Path(linker) / "link.exe").is_file()
print("native Windows registry/SDK paths verified; VS absence remains a valid empty result")
