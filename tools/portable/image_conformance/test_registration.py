"""Regressions for required Jacket IDs and actual passing program outcomes."""

import copy
import unittest
from check_registration import REQUIRED, validate_events


class RegistrationTests(unittest.TestCase):
    def events(self):
        rows = []
        for name in sorted(REQUIRED):
            rows += [
                {"type": "test", "event": event, "kind": "golden", "name": name}
                for event in ("started", "passed")
            ]
        return [*rows, {"type": "run", "event": "finished", "passed": len(REQUIRED)}]

    def test_complete_actual_run_is_accepted(self):
        validate_events(self.events(), catalogue=False)

    def test_empty_or_partial_run_is_not_success(self):
        for events in ([], self.events()[2:], self.events()[:-1]):
            with self.assertRaises(ValueError):
                validate_events(events, catalogue=False)

    def test_skipped_flaky_and_xfail_outcomes_remain_failures(self):
        for event in ("skipped", "flaky", "xfail", "xpass", "failed", "timed_out"):
            events = copy.deepcopy(self.events())
            events[1]["event"] = event
            with self.assertRaises(ValueError):
                validate_events(events, catalogue=False)

    def test_duplicate_or_wrong_ids_are_rejected(self):
        events = self.events()
        events[1]["name"] = "not-the-required-program"
        with self.assertRaises(ValueError):
            validate_events(events, catalogue=False)

    def test_catalogue_requires_each_id_as_a_golden_program(self):
        rows = [
            {"type": "list", "kind": "golden", "name": name}
            for name in sorted(REQUIRED)
        ]
        validate_events(rows, catalogue=True)
        for changed in (
            [],
            rows[:-1],
            [*rows, rows[0]],
            [dict(row, kind="test") for row in rows],
        ):
            with self.assertRaises(ValueError):
                validate_events(changed, catalogue=True)


if __name__ == "__main__":
    unittest.main()
