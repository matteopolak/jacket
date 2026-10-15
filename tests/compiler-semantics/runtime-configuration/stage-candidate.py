#!/usr/bin/env python3
"""Stage a private runtime-protocol integration candidate, writing only own target/."""
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[3]
# Full integration uses this checkout's canonical NEW library. The isolated
# compiler snapshot uses its explicitly assembled independent overlay.
SOURCE = ROOT / "stdlib"
if not (SOURCE / "prelude/module.jai").is_file():
    SOURCE = ROOT / "target/cleanroom-test-stdlib"
DESTINATION = ROOT / "target/runtime-configuration-candidate/stdlib"


def replace(text: str, old: str, new: str, count: int = 1) -> str:
    if text.count(old) != count:
        raise RuntimeError(f"NEW producer protocol changed; expected {count} instances of {old!r}")
    return text.replace(old, new)


def main() -> None:
    if not SOURCE.is_dir():
        raise SystemExit("assemble the independent test library first")
    if DESTINATION.exists():
        shutil.rmtree(DESTINATION)
    shutil.copytree(SOURCE, DESTINATION)

    path = DESTINATION / "Runtime_Support/module.jai"
    text = path.read_text()
    legacy_transport = "TEMPORARY_STORAGE_SIZE" in text
    if legacy_transport:
        text = replace(text, ", TEMPORARY_STORAGE_SIZE: s64 = 32768", "")
        text = replace(text, '#import "internal/runtime"(TEMPORARY_STORAGE_SIZE = TEMPORARY_STORAGE_SIZE)', '#import "internal/runtime"')
        text = replace(text, ", TEMPORARY_STORAGE_SIZE = TEMPORARY_STORAGE_SIZE", "")
        path.write_text(text)

    path = DESTINATION / "internal/runtime/startup.jai"
    text = path.read_text()
    if legacy_transport:
        text = replace(text, ", TEMPORARY_STORAGE_SIZE: s64 = 32768", "")
        text = replace(text, '#import "internal/runtime"(TEMPORARY_STORAGE_SIZE = TEMPORARY_STORAGE_SIZE)', '#import "internal/runtime"')
        path.write_text(text)

    path = DESTINATION / "internal/runtime/module.jai"
    text = path.read_text()
    if legacy_transport:
        text = replace(text, "#module_parameters(TEMPORARY_STORAGE_SIZE: s64 = 32768);\n#assert TEMPORARY_STORAGE_SIZE >= 0;\n", "")
        text = replace(text, "    storage.* = .{};", "    temporary_size := Runtime_Config.temporary_storage_size();\n\n    storage.* = .{};")
        text = replace(text, "TEMPORARY_STORAGE_SIZE", "temporary_size", 3)
        text += '\n#scope_file\nRuntime_Config :: #import "internal/compiler-api/runtime-config";\n'
        path.write_text(text.lstrip("\n"))
    elif '"internal/compiler-api/runtime-config"' not in text:
        raise RuntimeError("runtime has neither the checked old transport nor the private configuration import")
    helper = DESTINATION / "internal/compiler-api/runtime-config.jai"
    shutil.copyfile(ROOT / "stdlib/internal/compiler-api/runtime-config.jai", helper)
    print(DESTINATION)


if __name__ == "__main__":
    main()
