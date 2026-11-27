"""Version comparison, release selection and the issue plan of check_pin_updates.py, on recorded data."""

import json
import unittest
from datetime import datetime, timezone

import check_pin_updates as pins
from toolkit.github import LookupFailed

NOW = datetime(2026, 11, 9, tzinfo=timezone.utc)


def flake_lock(day):
    """A flake.lock whose rust-overlay input was locked at the start of `day` (UTC)."""
    stamp = int(datetime.fromisoformat(day).replace(tzinfo=timezone.utc).timestamp())
    return json.dumps({"nodes": {"rust-overlay": {"locked": {"lastModified": stamp}}}})


def release(tag, published="2026-09-01T00:00:00Z", **extra):
    return {"tag_name": tag, "published_at": published, "draft": False, "prerelease": False, **extra}


MANIFEST = {"date": "2026-10-01", "pkg": {"rust": {"version": "1.96.0 (abc 2026-09-30)"}}}
TOOLCHAIN = '[toolchain]\nchannel = "nightly-2026-08-29"\n'
PINS = {"llvm": "22.1.0", "just": "1.58.0", "nextest": "0.9.148"}
CURRENT = {
    "llvm": [release("llvmorg-22.1.0")],
    "just": [release("1.58.0")],
    "nextest": [release("cargo-nextest-0.9.148")],
}


KNOWN_LOCK = flake_lock("2026-09-01")


class VersionTests(unittest.TestCase):
    def test_numbers_are_compared_not_text(self):
        self.assertTrue(pins.newer("0.9.100", "0.9.99"))
        self.assertTrue(pins.newer("1.10.0", "1.9.9"))
        self.assertFalse(pins.newer("1.58.0", "1.58.0"))
        self.assertFalse(pins.newer("22.1.0", "23.1.10"))

    def test_a_longer_version_is_newer_than_its_prefix(self):
        self.assertTrue(pins.newer("1.2.1", "1.2"))

    def test_non_releases_are_rejected(self):
        for text in ("v1.2.3", "1.2.3-rc1", "", "1..2"):
            with self.assertRaises(ValueError, msg=text):
                pins.parse_version(text)


class LatestReleaseTests(unittest.TestCase):
    def test_picks_the_highest_version_not_the_newest_publication(self):
        releases = [release("1.2.0", "2026-09-02T00:00:00Z"), release("1.10.0", "2026-09-01T00:00:00Z"),
                    release("1.9.5", "2026-09-03T00:00:00Z")]
        self.assertEqual(pins.latest_release(releases, "", NOW), "1.10.0")

    def test_skips_prereleases_drafts_other_tags_and_young_releases(self):
        releases = [release("llvmorg-24.1.0", prerelease=True), release("llvmorg-24.0.1", draft=True),
                    release("llvmorg-24.1.0-rc1"), release("nextest-runner-9.9.9"),
                    release("llvmorg-24.1.0", "2026-11-01T00:00:00Z"), release("llvmorg-23.1.3")]
        self.assertEqual(pins.latest_release(releases, "llvmorg-", NOW), "23.1.3")

    def test_none_when_nothing_qualifies(self):
        self.assertIsNone(pins.latest_release([release("v1")], "", NOW))


class RustTests(unittest.TestCase):
    def test_reads_the_channel(self):
        self.assertEqual(pins.rust_pin(TOOLCHAIN), "nightly-2026-08-29")

    def test_a_nightly_before_the_stable_release_is_behind(self):
        row = pins.rust_row("nightly-2026-08-29", MANIFEST, NOW)
        self.assertEqual((row.name, row.pinned), ("rust", "nightly-2026-08-29"))
        self.assertIn("1.96.0", row.latest)

    def test_a_nightly_after_the_stable_release_is_current(self):
        self.assertIsNone(pins.rust_row("nightly-2026-10-01", MANIFEST, NOW))

    def test_a_young_stable_release_is_not_offered(self):
        young = {"date": "2026-11-05", "pkg": {"rust": {"version": "1.97.0 (x 2026-11-04)"}}}
        self.assertIsNone(pins.rust_row("nightly-2026-08-29", young, NOW))

    def test_a_version_channel_is_compared_by_number(self):
        self.assertIsNotNone(pins.rust_row("1.95.0", MANIFEST, NOW))
        self.assertIsNone(pins.rust_row("1.96.0", MANIFEST, NOW))


class OverlayTests(unittest.TestCase):
    def test_a_lock_from_after_the_nightly_knows_it(self):
        self.assertIsNone(pins.overlay_row("nightly-2026-11-13", flake_lock("2026-11-14")))
        self.assertIsNone(pins.overlay_row("nightly-2026-11-13", flake_lock("2026-12-01")))

    def test_a_lock_from_before_or_on_the_nightly_may_not_know_it(self):
        for day in ("2026-11-12", "2026-11-13"):
            row = pins.overlay_row("nightly-2026-11-13", flake_lock(day))
            self.assertEqual((row.name, row.where), ("rust-overlay", "flake.lock"), day)
            self.assertIn(day, row.pinned)
            self.assertIn("nightly-2026-11-13", row.latest)

    def test_a_version_channel_needs_no_check(self):
        self.assertIsNone(pins.overlay_row("1.96.0", flake_lock("2020-01-01")))

    def test_a_lock_without_rust_overlay_is_an_error(self):
        with self.assertRaises(LookupFailed):
            pins.overlay_row("nightly-2026-11-13", json.dumps({"nodes": {}}))

    def test_it_is_listed_with_the_other_pins(self):
        current = {"date": "2026-08-29", "pkg": {"rust": {"version": "1.95.0 (x y)"}}}
        rows = pins.outdated(PINS, TOOLCHAIN, current, CURRENT, NOW, flake_lock("2026-08-01"))
        self.assertEqual([r.name for r in rows], ["rust-overlay"])


class OutdatedTests(unittest.TestCase):
    def test_only_the_pins_behind_are_listed(self):
        releases = {**CURRENT, "just": [release("1.59.0")], "nextest": [release("cargo-nextest-0.9.149")]}
        rows = pins.outdated(PINS, TOOLCHAIN, MANIFEST, releases, NOW, KNOWN_LOCK)
        self.assertEqual([r.name for r in rows], ["rust", "just", "nextest"])
        self.assertEqual(rows[1].latest, "1.59.0")

    def test_everything_current(self):
        current = {"date": "2026-08-29", "pkg": {"rust": {"version": "1.95.0 (x y)"}}}
        self.assertEqual(pins.outdated(PINS, TOOLCHAIN, current, CURRENT, NOW, KNOWN_LOCK), [])

    def test_a_repository_with_no_usable_release_is_an_error(self):
        with self.assertRaises(LookupFailed):
            pins.outdated(PINS, TOOLCHAIN, MANIFEST, {**CURRENT, "llvm": []}, NOW, KNOWN_LOCK)


class PlanTests(unittest.TestCase):
    ROWS = [pins.Row("just", "1.58.0", "1.59.0", ".github/pins.json")]

    def issue(self, state, body=None):
        return {"number": 7, "state": state, "body": pins.issue_body(self.ROWS) if body is None else body}

    def test_body_lists_each_row_and_carries_the_marker(self):
        body = pins.issue_body(self.ROWS)
        self.assertIn(pins.MARKER, body)
        self.assertIn("| just | `1.58.0` | 1.59.0 |", body)
        self.assertEqual(body, pins.issue_body(self.ROWS))

    def test_create_when_there_is_no_issue(self):
        self.assertEqual(pins.plan(None, self.ROWS)[0], "create")

    def test_nothing_when_the_open_issue_already_says_it(self):
        self.assertEqual(pins.plan(self.issue("OPEN"), self.ROWS)[0], "nothing")

    def test_edit_when_the_list_changed(self):
        self.assertEqual(pins.plan(self.issue("OPEN", "old"), self.ROWS)[0], "edit")

    def test_reopen_a_closed_issue_instead_of_opening_a_second(self):
        self.assertEqual(pins.plan(self.issue("CLOSED"), self.ROWS)[0], "reopen")

    def test_close_when_current(self):
        self.assertEqual(pins.plan(self.issue("OPEN"), [])[0], "close")
        self.assertEqual(pins.plan(self.issue("CLOSED"), [])[0], "nothing")
        self.assertEqual(pins.plan(None, [])[0], "nothing")


if __name__ == "__main__":
    unittest.main()
