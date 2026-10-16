"""Browser dispatch consumes the reviewed catalogue before compiler execution."""

import json
from stdlib_test_catalog import (
    ROOT,
    case_for_mode,
    unavailable_reason,
    validated_entries,
)

BROWSER_MODE = "wasm-interp"


def browser_cases(root=ROOT):
    result = []
    for original in validated_entries(root):
        case = case_for_mode(original, BROWSER_MODE)
        reason = unavailable_reason(original, BROWSER_MODE)
        result.append(
            {
                **case,
                "path": case["path"].relative_to(root).as_posix(),
                "browser_unavailable_reason": reason,
            }
        )
    return result


if __name__ == "__main__":
    print(json.dumps(browser_cases()))
