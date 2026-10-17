"""Browser dispatch consumes the reviewed catalogue before compiler execution."""

import json
from stdlib_test_catalog import (
    ROOT,
    case_for_mode,
    unavailable_reason,
    validated_entries,
)

BROWSER_MODE = "wasm-interp"
# Compiler-host capability, not target-OS policy: a native compiler emitting WASM can have a clock
# even though the browser-hosted compiler cannot. A test requests one explicitly with a
# `required_browser_host_capabilities` manifest row; diagnostics never select it.
BROWSER_HOST_CAPABILITIES = frozenset()


def browser_cases(root=ROOT):
    result = []
    for original in validated_entries(root):
        case = case_for_mode(original, BROWSER_MODE)
        reason = unavailable_reason(original, BROWSER_MODE)
        required_host = sorted(set(case.get("required_browser_host_capabilities", ())))
        missing_host = set(required_host) - BROWSER_HOST_CAPABILITIES
        if reason is None and missing_host:
            reason = (
                "browser compiler host unavailable: requires "
                + ", ".join(sorted(missing_host))
                + "; native compiler execution remains required, not passed"
            )
        result.append(
            {
                **case,
                "path": case["path"].relative_to(root).as_posix(),
                "browser_unavailable_reason": reason,
                "required_browser_host_capabilities": required_host,
            }
        )
    return result


if __name__ == "__main__":
    print(json.dumps(browser_cases()))
