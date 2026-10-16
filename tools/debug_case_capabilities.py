"""Exact Debug fixture requirements; missing providers remain native failures."""


def apply_debug_capabilities(case):
    if case["id"] != "Debug/tests/ignore-ownership":
        return
    case["unsupported_modes"] = sorted(
        set(case.get("unsupported_modes", ())) | {"wasm-interp", "wasm-native"}
    )
    case["required_capabilities"] = list(
        dict.fromkeys(
            [
                *case.get("required_capabilities", ()),
                "native Debug diagnostic output and retained assertion state",
            ]
        )
    )
    case["required_os"] = ("macos", "linux", "windows")
