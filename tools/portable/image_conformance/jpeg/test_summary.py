"""Check how the JPEG gate names a case's outcome and folds rows into a status."""

import unittest

import run


def row(**fields):
    base = {
        "case": "a.jpg",
        "sha256": "00",
        "class": "valid",
        "mode": "native-o2",
        "accepted": True,
        "sanitizer_diagnostic": False,
        "result_observed": True,
        "process": {"exit": 0},
        "reference": {"clean": True},
        "failures": [],
        "review": [],
        "disposition": None,
        "recovery_pixel_mismatch": False,
    }
    return {**base, **fields}


CASE = {"id": "a.jpg", "sha256": "00", "unsupported_process": False}


def outcome(row, failures=(), review=(), exit=0, above_cap=()):
    return run.row_outcome(
        row, CASE, {"exit": exit}, list(failures), list(review), above_cap
    )


class RowOutcome(unittest.TestCase):
    def test_a_clean_acceptance(self):
        self.assertEqual(outcome(row()), "accepted-clean-reference")

    def test_an_acceptance_next_to_a_warning_bearing_reference(self):
        self.assertEqual(
            outcome(row(reference={"clean": False})), "accepted-damaged-or-unverified"
        )

    def test_a_crashed_probe_is_an_error_even_with_failures(self):
        self.assertEqual(outcome(row(), failures=["x"], exit=1), "error")

    def test_a_sanitizer_report_is_an_error(self):
        self.assertEqual(outcome(row(sanitizer_diagnostic=True)), "error")

    def test_failures_come_before_review(self):
        self.assertEqual(outcome(row(), failures=["x"], review=["y"]), "fail")

    def test_the_reviewed_invalid_acceptance_note_does_not_block(self):
        note = "reviewed-invalid-class-acceptance:x"
        self.assertEqual(outcome(row(), review=[note]), "accepted-clean-reference")
        self.assertEqual(outcome(row(), review=[note, "other"]), "review")

    def test_rejections_are_told_apart(self):
        rejected = row(accepted=False)
        self.assertEqual(outcome(rejected, above_cap=["a.jpg"]), "rejected-above-cap")
        self.assertEqual(outcome(rejected), "rejected-structural-or-malformed")
        external = row(
            accepted=False, disposition={"classification": "external-context-required"}
        )
        self.assertEqual(outcome(external), "rejected-external-context-required")


class Summarize(unittest.TestCase):
    def rows(self, modes):
        return [row(mode=mode, outcome="accepted-clean-reference") for mode in modes]

    def test_all_three_modes_pass(self):
        modes = list(run.MODES)
        summary = run.summarize(self.rows(modes), [CASE], modes, {})
        self.assertEqual(summary["status"], "PASS")
        self.assertEqual(summary["modes"]["native-o2"]["accepted"], 1)

    def test_a_focused_run_is_partial(self):
        summary = run.summarize(self.rows(["native-o2"]), [CASE], ["native-o2"], {})
        self.assertEqual(summary["status"], "PARTIAL")

    def test_a_failure_fails_the_gate(self):
        modes = list(run.MODES)
        rows = self.rows(modes)
        rows[0]["failures"] = ["x"]
        self.assertEqual(run.summarize(rows, [CASE], modes, {})["status"], "FAIL")

    def test_modes_that_disagree_fail_the_gate(self):
        modes = list(run.MODES)
        rows = self.rows(modes)
        rows[1]["accepted"] = False
        summary = run.summarize(rows, [CASE], modes, {})
        self.assertEqual(summary["cross_mode_status_differences"], ["a.jpg"])
        self.assertEqual(summary["status"], "FAIL")

    def test_reviewed_dispositions_change_the_pass_name(self):
        modes = list(run.MODES)
        summary = run.summarize(self.rows(modes), [CASE], modes, {"a.jpg": {}})
        self.assertEqual(summary["status"], "PASS_WITH_REVIEWED_SCOPE")


if __name__ == "__main__":
    unittest.main()
