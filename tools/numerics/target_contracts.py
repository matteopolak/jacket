"""Reviewed target contracts for the numerics runner; availability is not a pass."""

from __future__ import annotations

from dataclasses import dataclass

# The generated oracle lives in the output directory; its contract is keyed by this name.
ORACLE = "target/numerics/numeric_oracle.jai"


@dataclass(frozen=True)
class TargetContract:
    available: bool
    reason: str
    alternative: str | None = None


GETRECT_CASES = frozenset(
    {
        "stdlib/Color_Interpolation/tests/extended.jai",
        "stdlib/Float16/tests/edges.jai",
        "stdlib/GetRect/tests/color-policy-hash.jai",
        "stdlib/GetRect/tests/ui.jai",
        "stdlib/GetRect/tests/ui-smoke.jai",
        "stdlib/GetRect/tests/ui-interaction.jai",
        "stdlib/GetRect/tests/render-callbacks.jai",
        "stdlib/GetRect/tests/default-draw-procs.jai",
        "stdlib/Simp/tests/canonical-rendering.jai",
        "stdlib/GetRect/tests/type-indicator.jai",
        "stdlib/GetRect/tests/decimal-precision.jai",
        "stdlib/GetRect/tests/text-capacity.jai",
        "stdlib/GetRect/tests/wheel-events.jai",
        "stdlib/GetRect/tests/keyboard-shortcuts.jai",
        "stdlib/GetRect/tests/cursor-callbacks.jai",
        "stdlib/GetRect/tests/custom-indicator-target.jai",
        "stdlib/GetRect/tests/font-required.jai",
        "stdlib/GetRect/tests/texture-loader-required.jai",
        "stdlib/GetRect/tests/text-capacity-assert.jai",
    }
)

PORTABLE_CASES = frozenset(
    {
        "stdlib/Float16/tests/core.jai",
        "stdlib/Color_Interpolation/tests/policies.jai",
        "stdlib/Math/tests/driven/augmented-generic.jai",
        "stdlib/Float16/tests/algebra.jai",
        "stdlib/Color_Interpolation/tests/precision-rounding.jai",
        "stdlib/Math/tests/normalization-ieee.jai",
        "stdlib/Math/tests/abs-ieee.jai",
        "stdlib/Float16/tests/half-overflow.jai",
        ORACLE,
        "stdlib/Math/tests/float16-edge-vectors.jai",
    }
)

NATIVE_CURSOR_CASES = frozenset(
    {
        "stdlib/GetRect/tests/native-cursors.jai",
        "stdlib/GetRect/tests/native-ui-cursors.jai",
    }
)

MODULE_CASES = NATIVE_CURSOR_CASES - {"stdlib/GetRect/tests/native-ui-cursors.jai"} | {
    "stdlib/GetRect/tests/native-clipboard.jai",
    "stdlib/GetRect/tests/clipboard-unavailable.jai",
    "stdlib/GetRect/tests/cursors-unavailable.jai",
}


def target_contract(source: str, target: str) -> TargetContract:
    """Classify an exact inventoried fixture; reject unreviewed cases/targets."""
    if target not in {"linux", "windows", "macos", "wasm"}:
        raise ValueError(f"uninventoried target: {target}")
    if source in PORTABLE_CASES:
        return TargetContract(True, "portable numerical contract")
    if source == "stdlib/GetRect/tests/clipboard-text-actions-linux.jai":
        return TargetContract(
            target == "linux",
            "real canonical Clipboard in a CI-owned private X11 display",
        )
    if source in GETRECT_CASES:
        if target == "wasm":
            return TargetContract(
                False,
                "GetRect requires canonical Input.Event/Window_Type; no reviewed WASM window handle",
            )
        return TargetContract(
            True, "reviewed native window types; caller callbacks supported"
        )
    if source == "stdlib/GetRect/tests/native-clipboard.jai":
        return TargetContract(
            target == "macos",
            "private named-pasteboard transport and shutdown are macOS-only",
            None
            if target == "macos"
            else "stdlib/GetRect/tests/clipboard-unavailable.jai",
        )
    if source in NATIVE_CURSOR_CASES:
        return TargetContract(
            target in {"macos", "windows"},
            "system cursor resources exist only on implemented Windows/macOS providers",
            None
            if target in {"macos", "windows"}
            else "stdlib/GetRect/tests/cursors-unavailable.jai",
        )
    if source == "stdlib/GetRect/tests/cursors-unavailable.jai":
        return TargetContract(
            target not in {"windows", "macos"}, "explicit unavailable-provider negative"
        )
    if source == "stdlib/GetRect/tests/clipboard-unavailable.jai":
        return TargetContract(
            target != "macos", "explicit unavailable named-pasteboard negative"
        )
    raise ValueError(f"uninventoried fixture: {source}")
