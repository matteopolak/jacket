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


def unavailable_reason(case, mode):
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
