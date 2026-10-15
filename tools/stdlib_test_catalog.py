"""Source catalogue for reviewed contracts and independent regression families.

This contains no expected-failure baseline. Every selected supported case runs;
compiler/API errors remain failures. Native-only families are reported separately
from WASM execution and must be exercised by native CI.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGRESSION_FAMILIES = {
    "compiler": "compiler-regressions",
    "extensions": "extensions",
    "native": "native-regressions",
    "graphics": "graphics-regressions",
}

# Inputs required by the unchanged independently authored fixture programs.
PROGRAM_INPUTS = {
    "extensions/io-dependency-workers": "scratch-file",
    "tests/portable-data-resource": "scratch-directory",
    "SDL/tests/sdl-rwops": "scratch-file",
    "Simp/tests/glyph-atlas": "font",
}
PLATFORM_REQUIREMENTS = {
    "compiler/socket-select-canonical-timeval": ("linux", "macos"),
    "extensions/io-uring-dependency-repro": "linux",
    "Input/tests/windows-device-decoding": "windows",
    "tests/portable-data-console-windows": "windows",
    "tests/portable-data-discovery-windows": "windows",
    "tests/portable-data-manifest-windows": "windows",
    "tests/portable-data-resource-windows": "windows",
}


def entries(root=ROOT):
    manifest = root / "tests/stdlib-contract/manifest.json"
    for case in json.loads(manifest.read_text()):
        yield {**case, "id": "contracts/" + case["name"],
               "path": root / "tests/stdlib-contract" / case["source"],
               "family": "contracts"}
    for family, directory in REGRESSION_FAMILIES.items():
        for path in sorted((root / "tests" / directory).glob("*.jai")):
            yield {"id": family + "/" + path.stem, "path": path, "family": family}
    stdlib = root / "stdlib"
    found = (set(stdlib.glob("*/tests/*.jai")) | set(stdlib.glob("Extensions/*/tests/*.jai"))
             | set((stdlib / "tests").glob("**/*.jai")))
    for path in sorted(found):
        relative = path.relative_to(stdlib)
        if "modules" not in relative.parts:
            yield {"id": relative.with_suffix("").as_posix(), "path": path, "family": "modules"}


def validated_entries(root=ROOT):
    result = list(entries(root))
    names = set()
    for case in result:
        if case["id"] in {"native/clipboard-round-trip", "native/macos-clipboard-utf8", "graphics/getrect-widgets-driven"}:
            case["clipboard_guard"] = True
        if case["id"] in PROGRAM_INPUTS:
            case["input"] = PROGRAM_INPUTS[case["id"]]
        if case["id"] in PLATFORM_REQUIREMENTS:
            case["required_os"] = PLATFORM_REQUIREMENTS[case["id"]]
        if case["id"] in names:
            raise ValueError("duplicate test ID: " + case["id"])
        names.add(case["id"])
        if not case["path"].is_file():
            raise ValueError("missing test source: " + str(case["path"]))
        if not case["path"].resolve().is_relative_to(root.resolve()):
            raise ValueError("test source leaves repository: " + str(case["path"]))
    if not result:
        raise ValueError("empty test catalogue")
    return result


def unavailable_reason(case, mode, actual_platform=None):
    target_os = "wasm" if mode.startswith("wasm") else (actual_platform or "").split("-")[0]
    required = case.get("required_os")
    required = [required] if isinstance(required, str) else required
    if required and target_os and target_os not in required:
        return f"requires actual {'/'.join(required)} host; native execution remains required, not passed on {target_os}"
    if mode.startswith("wasm") and case["family"] in {"native", "graphics"}:
        return "native SDK/OS/graphics regression requires native CI; no WASM runtime claim"
    return None


def cases(root=ROOT):
    for case in validated_entries(root):
        yield case["id"], case["path"]


if __name__ == "__main__":
    # The browser runner consumes the same reviewed manifest and source discovery.
    print(json.dumps([{**case, "path": str(case["path"].relative_to(ROOT))}
                      for case in validated_entries()]))
