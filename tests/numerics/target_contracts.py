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
        "tests/numerics/extended.jai",
        "tests/numerics/edges.jai",
        "tests/numerics/color_policies.jai",
        "tests/numerics/ui.jai",
        "tests/numerics/ui_smoke.jai",
        "tests/numerics/ui_interaction.jai",
        "tests/numerics/render_callbacks.jai",
        "tests/numerics/default_draw_procs.jai",
        "tests/numerics/canonical_rendering.jai",
        "tests/numerics/type_indicator.jai",
        "tests/numerics/decimal_precision.jai",
        "tests/numerics/text_capacity.jai",
        "tests/numerics/wheel_events.jai",
        "tests/numerics/keyboard_shortcuts.jai",
        "tests/numerics/cursor_callbacks.jai",
        "tests/numerics/custom_indicator_target_reproducer.jai",
        "tests/numerics/font_required.jai",
        "tests/numerics/texture_loader_required.jai",
        "tests/numerics/text_capacity_assert.jai",
    }
)

PORTABLE_CASES = frozenset(
    {
        "tests/numerics/core.jai",
        "tests/numerics/color_policies_portable.jai",
        "tests/numerics/augmented_generic_reproducer.jai",
        "tests/numerics/algebra.jai",
        "tests/numerics/precision_rounding.jai",
        "tests/numerics/normalization_ieee.jai",
        "tests/numerics/abs_ieee.jai",
        "tests/numerics/half_overflow.jai",
        ORACLE,
        "tests/stdlib-contract/math-float16/main.jai",
    }
)

NATIVE_CURSOR_CASES = frozenset(
    {
        "stdlib/GetRect/tests/native_cursors.jai",
        "tests/numerics/native_ui_cursors.jai",
    }
)

MODULE_CASES = NATIVE_CURSOR_CASES - {"tests/numerics/native_ui_cursors.jai"} | {
    "stdlib/GetRect/tests/native_clipboard.jai",
    "stdlib/GetRect/tests/clipboard_unavailable.jai",
    "stdlib/GetRect/tests/cursors_unavailable.jai",
}


def target_contract(source: str, target: str) -> TargetContract:
    """Classify an exact inventoried fixture; reject unreviewed cases/targets."""
    if target not in {"linux", "windows", "macos", "wasm"}:
        raise ValueError(f"uninventoried target: {target}")
    if source in PORTABLE_CASES:
        return TargetContract(True, "portable numerical contract")
    if source == "tests/numerics/clipboard_text_actions_linux.jai":
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
    if source == "stdlib/GetRect/tests/native_clipboard.jai":
        return TargetContract(
            target == "macos",
            "private named-pasteboard transport and shutdown are macOS-only",
            None
            if target == "macos"
            else "stdlib/GetRect/tests/clipboard_unavailable.jai",
        )
    if source in NATIVE_CURSOR_CASES:
        return TargetContract(
            target in {"macos", "windows"},
            "system cursor resources exist only on implemented Windows/macOS providers",
            None
            if target in {"macos", "windows"}
            else "stdlib/GetRect/tests/cursors_unavailable.jai",
        )
    if source == "stdlib/GetRect/tests/cursors_unavailable.jai":
        return TargetContract(
            target not in {"windows", "macos"}, "explicit unavailable-provider negative"
        )
    if source == "stdlib/GetRect/tests/clipboard_unavailable.jai":
        return TargetContract(
            target != "macos", "explicit unavailable named-pasteboard negative"
        )
    raise ValueError(f"uninventoried fixture: {source}")
