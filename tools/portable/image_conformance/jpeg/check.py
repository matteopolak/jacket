#!/usr/bin/env python3
"""Check pinned inventory and gate-policy invariants without running a compiler or decoder."""

import importlib.util
import json
from pathlib import Path
import tempfile


def main():
    root = Path(__file__).resolve().parent
    manifest = json.loads((root / "manifest.json").read_text())
    assert len(manifest["cases"]) == 258
    assert len({case["id"] for case in manifest["cases"]}) == 258
    sources = {entry["local_path"]: entry for entry in manifest["sources"]}
    assert len(sources) == 296
    for case in manifest["cases"]:
        assert sources[case["id"]]["sha256"] == case["sha256"]
        assert sources[case["id"]]["bytes"] == case["bytes"]
    spec = importlib.util.spec_from_file_location("jpeg_gate", root / "run.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    assert module.result_match("RESULT false 0 0 0 rejected") is not None
    for text in (
        "",
        "RESULT false",
        "RESULT false 0 0 0 rejected\nRESULT false 0 0 0 rejected",
        "RESULT false 0 0 0 rejected\nRESULT malformed",
    ):
        assert module.result_match(text) is None
    assert not module.decision_known(
        {"process": {"exit": 1}, "accepted": False, "result_observed": False}
    )
    assert not module.decision_known(
        {"process": {"exit": 0}, "accepted": False, "result_observed": False}
    )
    assert module.decision_known(
        {"process": {"exit": 0}, "accepted": False, "result_observed": True}
    )
    actual = json.loads((root / "reviewed-dispositions.json").read_text())
    reviewed = module.review_map(root / "reviewed-dispositions.json", manifest["cases"])
    assert set(reviewed) == {case["id"] for case in actual["cases"]}
    for pending in actual["unresolved_cases"]:
        assert pending["id"] not in reviewed
    mjpeg = reviewed["codec-corpus/jpeg-conformance/valid/mjpeg.jpg"]
    assert mjpeg["expectation"] == "reject"
    assert mjpeg["classification"] == "external-context-required"
    assert (
        reviewed[
            "codec-corpus/jpeg-conformance/invalid/316be81dfdeeb942e904feb3a77f4f83.jpg"
        ]["expectation"]
        == "retained-compatibility-extension"
    )
    case = {
        "id": "example",
        "class": "valid",
        "expectation": "accept",
        "unsupported_process": False,
    }
    row = {
        "process": {"exit": 0},
        "sanitizer_diagnostic": False,
        "parity": True,
        "accepted": True,
        "result_observed": True,
        "invariant": True,
        "cap_minus_one": True,
        "cap_exact": True,
        "above_cap_ids": [],
        "pixels": {"pass": False},
    }
    missing = dict(row, accepted=False, result_observed=False, reject_empty=True)
    assert (
        "decoder-result-missing-or-ambiguous"
        in module.classify(
            dict(case, expectation="reject"),
            missing,
            {"clean": False, "exit": 1},
            None,
        )[0]
    )
    clean = {"exit": 0, "clean": True, "warning_regression": False}
    damaged = dict(clean, clean=False)
    recovery = {"expectation": "documented-recovery"}
    assert (
        "clean-reference-pixel-mismatch" in module.classify(case, row, clean, None)[0]
    )
    assert (
        "clean-reference-pixel-mismatch"
        in module.classify(case, row, clean, recovery)[0]
    )
    assert (
        "damaged-recovery-pixel-mismatch"
        in module.classify(case, row, damaged, None)[1]
    )
    assert not module.classify(case, row, damaged, recovery)[1]
    assert (
        "independent-reference-warning-change"
        in module.classify(case, row, dict(damaged, warning_regression=True), recovery)[
            0
        ]
    )
    compatibility = {"expectation": "retained-compatibility-extension"}
    assert (
        "clean-reference-pixel-mismatch"
        in module.classify(case, row, clean, compatibility)[0]
    )
    assert (
        "required-acceptance-rejected"
        in module.classify(
            case, dict(row, accepted=False, reject_empty=True), damaged, compatibility
        )[0]
    )
    invalid = dict(case, **{"class": "invalid", "expectation": "reject"})
    assert (
        "strict-required-rejection-accepted"
        in module.classify(invalid, row, damaged, None)[0]
    )
    assert (
        "sanitizer-diagnostic"
        in module.classify(
            case, dict(row, sanitizer_diagnostic=True), damaged, recovery
        )[0]
    )
    with tempfile.TemporaryDirectory() as directory:
        actual = Path(directory) / "actual"
        reference = Path(directory) / "reference"
        actual.write_bytes(bytes([255, 128, 0]))
        reference.write_bytes(bytes([0, 127, 255, 0]))
        result = module.pixel_compare(
            actual, reference, [1, 1, 4, 0, 4], {"width": 1, "height": 1, "channels": 3}
        )
        assert result["pass"] and result["max_error"] == 0
    print(
        "258-case inventory and policy invariants passed; no compiler/decoder execution."
    )


if __name__ == "__main__":
    main()
