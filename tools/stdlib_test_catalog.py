"""Source catalogue for reviewed contracts and independent regression families.

This contains no expected-failure baseline. Every selected supported case runs;
compiler/API errors remain failures. Native-only families are reported separately
from WASM execution and must be exercised by native CI.
"""

import hashlib
import json
from pathlib import Path

from case_capabilities import user_deferred_reason
from platform_io_capabilities import RUNTIME_CONTRACT_CAPABILITIES
from foundations_case_capabilities import apply_foundations_capabilities
from debug_case_capabilities import apply_debug_capabilities
from numerics_case_capabilities import (
    apply_numerics_capabilities,
    numerics_unavailable_reason,
)

ROOT = Path(__file__).resolve().parents[1]


def reference_rejected_reason(case):
    if (
        case["id"] == "compiler/proc-type-defaults-ifx"
        and hashlib.sha256(case["path"].read_bytes()).hexdigest()
        == "e2aaa5f6c468e62fb635f48ca5ada3532abeba02f8f7e2388cfec634f5108968"
    ):
        return "root official compiler rejects the original overloaded/concrete ifx mismatch; not a supported positive, regardless of raw local acceptance"
    return None


REGRESSION_FAMILIES = {
    "compiler": "compiler-regressions",
    "extensions": "extensions",
    "native": "native-regressions",
    "graphics": "graphics-regressions",
}

# Reviewed HTTP test inventory. Exact IDs make missing standalone regressions
# fail dispatch rather than disappear behind a successful substring filter.
HTTP_CONFORMANCE_CASE_IDS = tuple(
    "Extensions/Http/tests/" + name
    for name in (
        "chunking",
        "client",
        "codec",
        "https",
        "llhttp",
        "server_connections",
        "server_extractors",
        "server_responses",
        "server_routing",
        "smuggling",
        "testing",
    )
) + ("extensions/http-codec-roundtrip",)

# These individual programs actually use native Io/fiber services: the Http app, router and
# groups are not part of the WebAssembly module. Pure codec programs deliberately have no
# such restriction.
HTTP_NATIVE_IO_CASE_IDS = frozenset(
    ["extensions/http-router-ownership"]
    + [
        "Extensions/Http/tests/" + name
        for name in (
            "client",
            "https",
            "server_connections",
            "server_extractors",
            "server_responses",
            "server_routing",
            "testing",
        )
    ]
)

# Inputs required by the unchanged independently authored fixture programs.
PROGRAM_INPUTS = {
    "extensions/io-dependency-workers": "scratch-file",
    "tests/portable-data-resource": "scratch-directory",
    "tests/portable-data-resource-windows": "windows-resource-directory",
    "SDL/tests/sdl-rwops": "scratch-file",
    "Simp/tests/glyph-atlas": "font",
}
PROGRAM_ARGUMENTS = {
    "Clipboard/tests/cocoa-operations": ["enabled"],
    "Simp/tests/context-default-assertions": ["enabled"],
}
PLATFORM_REQUIREMENTS = {
    "Clipboard/tests/cocoa-operations": "macos",
    "Simp/tests/context-default-assertions": "macos",
    "Window_Creation/tests/macos-window": "macos",
    "Window_Creation/tests/macos-events": "macos",
    "native/macos-clipboard-utf8": "macos",
    "native/objc-window-levels": "macos",
    "native/posix-stat-and-mutex": ("linux", "macos"),
    "native/proc-sentinel-constant": ("linux", "macos"),
    # ImGui formats through the libffi varargs bridge, which no Windows job builds.
    "native/imgui-headless-frames": ("linux", "macos"),
    "graphics/macos-window-context-reopen": "macos",
    "graphics/input-multi-window-focus": "macos",
    "graphics/input-published-events": "macos",
    "graphics/input-temporary-allocator": "macos",
    "compiler/socket-select-canonical-timeval": ("linux", "macos"),
    "extensions/io-uring-dependency-repro": "linux",
    "Input/tests/windows-device-decoding": "windows",
    "Print_Color/tests/windows-console": "windows",
    "tests/portable-data-console-windows": "windows",
    "tests/portable-data-discovery-windows": "windows",
    "tests/portable-data-manifest-windows": "windows",
    "tests/portable-data-resource-windows": "windows",
}


def entries(root=ROOT):
    manifest = root / "tests/stdlib-contract/manifest.json"
    for case in json.loads(manifest.read_text()):
        yield {
            **case,
            "id": "contracts/" + case["name"],
            "path": root / "tests/stdlib-contract" / case["source"],
            "family": "contracts",
        }
    for family, directory in REGRESSION_FAMILIES.items():
        for path in sorted((root / "tests" / directory).glob("*.jai")):
            if not any(part.startswith("._") for part in path.relative_to(root).parts):
                yield {"id": family + "/" + path.stem, "path": path, "family": family}
    stdlib = root / "stdlib"
    found = (
        set(stdlib.glob("*/tests/*.jai"))
        | set(stdlib.glob("Extensions/*/tests/*.jai"))
        | set((stdlib / "tests").glob("**/*.jai"))
    )
    for path in sorted(found):
        relative = path.relative_to(stdlib)
        if "modules" not in relative.parts and not any(
            part.startswith("._") for part in relative.parts
        ):
            yield {
                "id": relative.with_suffix("").as_posix(),
                "path": path,
                "family": "modules",
            }


def validated_entries(root=ROOT):
    result = list(entries(root))
    names = set()
    for case in result:
        if case["id"] in RUNTIME_CONTRACT_CAPABILITIES:
            case["unsupported_modes"] = ["wasm-interp", "wasm-native"]
            case["required_capabilities"] = list(
                RUNTIME_CONTRACT_CAPABILITIES[case["id"]]
            )
        if case["id"] in HTTP_NATIVE_IO_CASE_IDS:
            case["unsupported_modes"] = ["wasm-interp", "wasm-native"]
            case["required_capabilities"] = [
                "native Io providers",
                "fiber context switching",
            ]
        apply_foundations_capabilities(case)
        apply_debug_capabilities(case)
        apply_numerics_capabilities(case)
        if case["id"] in {
            "native/clipboard-round-trip",
            "native/macos-clipboard-utf8",
        }:
            case["clipboard_guard"] = True
        termination = {
            "extensions/http-wasm-termination-status": (37, "before exit 37\n"),
            "extensions/http-wasm-termination-failure": (1, "FAIL: independent WASM termination control\n1 failure(s)\n"),
        }.get(case["id"])
        if termination:
            case.update(exit_code=termination[0], stdout=termination[1], stderr="")
        if case["family"] == "graphics" or case["id"] in PLATFORM_REQUIREMENTS and "macos" in PLATFORM_REQUIREMENTS[case["id"]]:
            case["window_lease"] = True
        if case["id"] in PROGRAM_INPUTS:
            case["input"] = PROGRAM_INPUTS[case["id"]]
        if case["id"] in PROGRAM_ARGUMENTS:
            case["arguments"] = PROGRAM_ARGUMENTS[case["id"]].copy()
        if case["id"] in PLATFORM_REQUIREMENTS:
            case["required_os"] = PLATFORM_REQUIREMENTS[case["id"]]
        if case["id"] == "Simp/tests/context-default-assertions":
            case["fixture_libraries"] = {"GL_PROBE": "window_gl_probe"}
            case["requires_display"] = True
        if case["id"] == "SDL/tests/sdl-wm":
            # Creates a real (hidden) SDL window, which needs a display server.
            case["requires_display"] = True
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


def case_for_mode(case, mode):
    """Return the explicit mode contract without mutating the discovered source.

    Selectors must use this result for dispatch AND expected-result accounting.
    A required negative is a real check with a precise diagnostic, never a pass
    inferred from target unavailability or an arbitrary compilation failure.
    """
    contract = case.get("mode_contracts", {}).get(mode, {})
    if contract and (
        set(contract) != {"mode", "diagnostic_contains"}
        or contract.get("mode") != "check-fail"
        or not isinstance(contract.get("diagnostic_contains"), str)
        or not contract["diagnostic_contains"].strip()
    ):
        raise ValueError("mode contract requires check-fail and a precise diagnostic")
    if contract and mode in case.get("unsupported_modes", ()):
        raise ValueError("required mode rejection conflicts with unavailable metadata")
    return {**case, **contract}


def unavailable_reason(case, mode, actual_platform=None):
    case = case_for_mode(case, mode)
    numerics_reason = numerics_unavailable_reason(case, mode, actual_platform)
    if numerics_reason:
        return numerics_reason
    if mode in case.get("unsupported_modes", []):
        required = ", ".join(case["required_capabilities"])
        return f"{mode} platform unavailable: requires {required}; actual native execution remains required, not passed"
    target_os = (
        "wasm" if mode.startswith("wasm") else (actual_platform or "").split("-")[0]
    )
    deferred = user_deferred_reason(case, target_os)
    if deferred:
        return deferred
    required = case.get("required_os")
    required = [required] if isinstance(required, str) else required
    if required and target_os and target_os not in required:
        return f"requires actual {'/'.join(required)} host; native execution remains required, not passed on {target_os}"
    target_cpu = (
        "wasm64"
        if mode.startswith("wasm")
        else (actual_platform or "").partition("-")[2].partition("-")[0]
    )
    required_cpu = case.get("required_cpu")
    if required_cpu and target_cpu and target_cpu != required_cpu:
        return f"requires actual {required_cpu} CPU; execution remains required, not passed on {target_cpu}"
    if mode.startswith("wasm") and case["family"] in {"native", "graphics"}:
        return "native SDK/OS/graphics regression requires native CI; no WASM runtime claim"
    return None


def cases(root=ROOT):
    for case in validated_entries(root):
        yield case["id"], case["path"]


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Project the canonical catalogue onto an explicit execution mode")
    parser.add_argument("--mode", choices=("interp", "native", "wasm-interp", "wasm-native"))
    parser.add_argument("--platform")
    arguments = parser.parse_args()
    entries = validated_entries()
    if arguments.mode:
        entries = [
            {**case_for_mode(case, arguments.mode),
             "selected_unavailable_reason": unavailable_reason(case, arguments.mode, arguments.platform)}
            for case in entries
        ]
    # The browser runner consumes the same reviewed manifest and source discovery.
    print(
        json.dumps(
            [
                {**case, "path": str(case["path"].relative_to(ROOT))}
                for case in entries
            ]
        )
    )
