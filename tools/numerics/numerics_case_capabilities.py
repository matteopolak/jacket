"""Connect the common catalogue to the reviewed numerics fixture contracts.

ASan and ordinary stdlib runners consume this same metadata before dispatch.
No compiler diagnostic or provider lookup participates in target selection.
"""

from functools import lru_cache
import importlib.util
from pathlib import Path
import sys


@lru_cache(maxsize=1)
def _contracts():
    path = Path(__file__).resolve().parents[1] / "tools/numerics/target_contracts.py"
    spec = importlib.util.spec_from_file_location("_jaic_numerics_contracts", path)
    module = importlib.util.module_from_spec(spec)
    # dataclasses resolves its defining module through sys.modules.
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def apply_numerics_capabilities(case):
    """Annotate only the explicitly inventoried module fixtures."""
    if case["family"] != "modules":
        return
    source = "stdlib/" + case["id"] + ".jai"
    if source in _contracts().MODULE_CASES:
        case["numerics_target_source"] = source


def numerics_unavailable_reason(case, mode, actual_platform):
    source = case.get("numerics_target_source")
    if source is None:
        return None
    contracts = _contracts()
    if source not in contracts.MODULE_CASES:
        raise ValueError("uninventoried numerics module fixture: " + source)
    if mode not in {"interp", "native", "wasm-interp", "wasm-native"}:
        raise ValueError("invalid numerics execution mode: " + mode)
    if mode.startswith("wasm-"):
        target = "wasm"
    else:
        parts = (actual_platform or "").split("-")
        if (
            len(parts) != 2
            or parts[0] not in {"linux", "macos", "windows"}
            or parts[1] not in {"x64", "arm64"}
        ):
            raise ValueError(f"invalid numerics native platform: {actual_platform!r}")
        target = parts[0]
    contract = contracts.target_contract(source, target)
    if contract.available:
        return None
    return f"{contract.reason}; fixture unavailable on {target}, not passed"
