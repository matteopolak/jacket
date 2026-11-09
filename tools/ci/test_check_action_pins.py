import importlib.util
import json
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "pins", Path(__file__).with_name("check_action_pins.py")
)
pins = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pins)

NOW = datetime(2026, 11, 8, tzinfo=timezone.utc)
OLD, NEW, NEWER, YOUNG, TAGOBJ = ("a" * 40, "b" * 40, "c" * 40, "d" * 40, "e" * 40)


def days_ago(days):
    return (NOW - timedelta(days=days)).isoformat().replace("+00:00", "Z")


def release(tag, days, **extra):
    return {
        "tag_name": tag,
        "published_at": days_ago(days),
        "draft": False,
        "prerelease": False,
        **extra,
    }


def ref(sha, kind="commit"):
    return {"object": {"type": kind, "sha": sha}}


class FakeApi:
    """Recorded answers by path; a path with no answer is a 404, like GitHub's."""

    def __init__(self, answers):
        self.answers, self.calls = answers, []

    def __call__(self, path):
        self.calls.append(path)
        if path not in self.answers:
            raise pins.NotFound(path)
        return self.answers[path]


RELEASES = "/repos/o/r/releases?per_page=100"


def tag_path(tag):
    return f"/repos/o/r/git/ref/tags/{tag}"


def api_for(releases, tags):
    answers = {RELEASES: releases}
    answers |= {tag_path(tag): ref(sha) for tag, sha in tags.items()}
    return FakeApi(answers)


def pin(sha=OLD, tag="v1.0.0", action="o/r"):
    return pins.Pin(".github/workflows/ci.yml", 7, "o/r", action, sha, tag)


class ParsingTests(unittest.TestCase):
    def test_finds_remote_pins_and_skips_local_ones(self):
        text = "\n".join(
            [
                f"      - uses: actions/checkout@{OLD} # v4.1.0",
                f"      - uses: o/r/sub/path@{NEW}   # v2 extra words",
                "      - uses: ./.github/actions/rust-cache",
                "      - uses: docker://alpine:3",
                f"      # uses: actions/old@{OLD} # v1",
                f"        uses: o/bare@{OLD}",
            ]
        )
        found, errors = pins.find_pins("f.yml", text)
        self.assertEqual(
            [(p.repo, p.action, p.sha, p.tag, p.line) for p in found],
            [
                ("actions/checkout", "actions/checkout", OLD, "v4.1.0", 1),
                ("o/r", "o/r/sub/path", NEW, "v2", 2),
                ("o/bare", "o/bare", OLD, None, 6),
            ],
        )
        self.assertEqual(errors, [])

    def test_unpinned_refs_are_errors(self):
        found, errors = pins.find_pins("f.yml", "- uses: actions/checkout@v4\n")
        self.assertEqual(found, [])
        self.assertIn("f.yml:1: actions/checkout@v4 is not pinned", errors[0])

    def test_version_ordering_ignores_unstable_tags(self):
        self.assertEqual(pins.version("v1.2.3"), (1, 2, 3))
        self.assertEqual(pins.version("v4"), (4, 0, 0))
        self.assertLess(pins.version("v1.9.0"), pins.version("v1.10.0"))
        for tag in ("v1.2.3-rc1", "nightly", "v1.2.3.4", "", None):
            self.assertIsNone(pins.version(tag))


class TagTests(unittest.TestCase):
    def test_annotated_tags_are_peeled_to_the_commit(self):
        api = FakeApi(
            {
                tag_path("v1"): ref(TAGOBJ, "tag"),
                f"/repos/o/r/git/tags/{TAGOBJ}": ref(NEW),
            }
        )
        self.assertEqual(pins.tag_commit(api, "o/r", "v1"), NEW)

    def test_missing_tag_is_none(self):
        self.assertIsNone(pins.tag_commit(FakeApi({}), "o/r", "v9"))

    def test_tag_pointing_at_a_blob_is_none(self):
        self.assertIsNone(
            pins.tag_commit(FakeApi({tag_path("v1"): ref(NEW, "blob")}), "o/r", "v1")
        )


class SelectionTests(unittest.TestCase):
    def test_newest_release_at_least_14_days_old(self):
        releases = [release("v1.3.0", 13), release("v1.2.0", 14), release("v1.1.0", 40)]
        api = api_for(releases, {"v1.2.0": NEW, "v1.1.0": OLD})
        self.assertEqual(
            pins.newest_eligible(api, "o/r", releases, NOW), ("v1.2.0", NEW)
        )

    def test_prereleases_drafts_and_odd_tags_are_skipped(self):
        releases = [
            release("v3.0.0", 30, prerelease=True),
            release("v2.5.0", 30, draft=True),
            release("nightly", 30),
            release("v2.0.0-rc.1", 30),
            release("v1.5.0", 30),
        ]
        api = api_for(releases, {"v1.5.0": NEW})
        self.assertEqual(
            pins.newest_eligible(api, "o/r", releases, NOW), ("v1.5.0", NEW)
        )

    def test_ordering_is_by_version_not_by_listing_or_date(self):
        releases = [
            release("v1.9.9", 20),
            release("v1.10.0", 30),
            release("v1.2.0", 15),
        ]
        api = api_for(releases, {"v1.10.0": NEW})
        self.assertEqual(
            pins.newest_eligible(api, "o/r", releases, NOW), ("v1.10.0", NEW)
        )

    def test_nothing_old_enough(self):
        releases = [release("v1.1.0", 1)]
        self.assertIsNone(
            pins.newest_eligible(api_for(releases, {}), "o/r", releases, NOW)
        )

    def test_release_whose_tag_is_gone_is_passed_over(self):
        releases = [release("v1.2.0", 30), release("v1.1.0", 40)]
        api = api_for(releases, {"v1.1.0": OLD})
        self.assertEqual(
            pins.newest_eligible(api, "o/r", releases, NOW), ("v1.1.0", OLD)
        )


class CheckTests(unittest.TestCase):
    def run_check(self, pin_list, releases, tags, policy_only=False):
        return pins.check(api_for(releases, tags), pin_list, NOW, policy_only)

    def test_current_pin_passes(self):
        errors, updates = self.run_check(
            [pin()], [release("v1.0.0", 30)], {"v1.0.0": OLD}
        )
        self.assertEqual((errors, updates), ([], []))

    def test_sha_that_is_not_the_commented_tag(self):
        errors, _ = self.run_check(
            [pin(sha=OLD)], [release("v1.0.0", 30)], {"v1.0.0": NEW}
        )
        self.assertEqual(len(errors), 1)
        self.assertIn("tag v1.0.0 points to bbbbbbbbbbbb", errors[0])
        self.assertIn(".github/workflows/ci.yml:7", errors[0])

    def test_missing_tag(self):
        errors, _ = self.run_check([pin()], [release("v1.0.0", 30)], {})
        self.assertIn("tag v1.0.0 does not exist", errors[0])

    def test_missing_comment(self):
        errors, updates = self.run_check([pin(tag=None)], [], {})
        self.assertIn("no version comment", errors[0])
        self.assertEqual(updates, [])

    def test_young_pin_is_an_error(self):
        errors, _ = self.run_check([pin()], [release("v1.0.0", 13)], {"v1.0.0": OLD})
        self.assertIn("younger than 14 days", errors[0])

    def test_age_of_a_tag_without_a_release_is_its_commit_date(self):
        api = api_for([], {"v1.0.0": OLD})
        api.answers[f"/repos/o/r/commits/{OLD}"] = {
            "commit": {"committer": {"date": days_ago(3)}}
        }
        errors, _ = pins.check(api, [pin()], NOW)
        self.assertIn("younger than 14 days", errors[0])

    def test_update_within_the_same_major(self):
        releases = [release("v1.4.0", 20), release("v1.0.0", 90)]
        _, updates = self.run_check([pin()], releases, {"v1.0.0": OLD, "v1.4.0": NEW})
        self.assertEqual(updates, [pins.Update("o/r", "v1.0.0", "v1.4.0", NEW, False)])

    def test_newer_major_is_reported_as_such(self):
        releases = [release("v2.0.0", 20), release("v1.0.0", 90)]
        _, updates = self.run_check([pin()], releases, {"v1.0.0": OLD, "v2.0.0": NEW})
        self.assertEqual(updates, [pins.Update("o/r", "v1.0.0", "v2.0.0", NEW, True)])
        self.assertIn("new major version", pins.describe(updates[0]))

    def test_release_too_young_is_not_offered(self):
        releases = [release("v1.4.0", 5), release("v1.0.0", 90)]
        _, updates = self.run_check([pin()], releases, {"v1.0.0": OLD, "v1.4.0": NEW})
        self.assertEqual(updates, [])

    def test_policy_only_skips_updates(self):
        releases = [release("v1.4.0", 20), release("v1.0.0", 90)]
        api = api_for(releases, {"v1.0.0": OLD})
        errors, updates = pins.check(api, [pin()], NOW, policy_only=True)
        self.assertEqual((errors, updates), ([], []))
        self.assertNotIn(tag_path("v1.4.0"), api.calls)

    def test_pins_of_one_repository_share_lookups(self):
        api = api_for([release("v1.0.0", 30)], {"v1.0.0": OLD})
        pins.check(api, [pin(), pin(action="o/r/sub")], NOW)
        self.assertEqual(api.calls.count(RELEASES), 1)


class RewriteTests(unittest.TestCase):
    update = pins.Update("o/r", "v1.0.0", "v1.4.0", NEW, False)

    def test_rewrites_sha_and_comment_everywhere_the_action_appears(self):
        text = (
            f"- uses: o/r@{OLD} # v1.0.0\n"
            f"- uses: o/r/sub@{OLD}   # v1.0.0 (kept note)\n"
            f"- uses: o/other@{OLD} # v1.0.0\n"
            f"- uses: o/r@{NEWER} # v1.2.0\n"
            "- uses: ./local\n"
        )
        new, changed = pins.rewrite(text, [self.update])
        self.assertEqual(changed, 2)
        self.assertEqual(
            new,
            f"- uses: o/r@{NEW} # v1.4.0\n"
            f"- uses: o/r/sub@{NEW}   # v1.4.0 (kept note)\n"
            f"- uses: o/other@{OLD} # v1.0.0\n"
            f"- uses: o/r@{NEWER} # v1.2.0\n"
            "- uses: ./local\n",
        )

    def test_rewritten_text_reads_back_as_the_new_pin(self):
        new, _ = pins.rewrite(f"- uses: o/r@{OLD} # v1.0.0\n", [self.update])
        found, errors = pins.find_pins("f.yml", new)
        self.assertEqual((found[0].sha, found[0].tag, errors), (NEW, "v1.4.0", []))


class CacheTests(unittest.TestCase):
    def setUp(self):
        self.fetched = []
        self.original = pins.fetch
        pins.fetch = self.fetch
        self.addCleanup(setattr, pins, "fetch", self.original)

    def fetch(self, url, **_kwargs):
        self.fetched.append(url)
        return b'{"ok": true}'

    def test_cached_answers_are_reused_until_they_expire(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "cache.json"

            def entry(age):
                return {"fetched": (NOW - age).isoformat(), "body": {"cached": True}}

            path.write_text(
                json.dumps(
                    {
                        "/fresh": entry(timedelta(hours=1)),
                        "/stale": entry(timedelta(hours=7)),
                    }
                )
            )
            api = pins.GitHub(path, NOW)
            self.assertEqual(api("/fresh"), {"cached": True})
            self.assertEqual(self.fetched, [])
            self.assertEqual(api("/stale"), {"ok": True})
            self.assertEqual(len(self.fetched), 1)
            api.save()
            self.assertEqual(pins.GitHub(path, NOW)("/stale"), {"ok": True})
            self.assertEqual(len(self.fetched), 1)

    def test_refresh_ignores_the_cache(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "cache.json"
            path.write_text(
                json.dumps(
                    {"/x": {"fetched": NOW.isoformat(), "body": {"cached": True}}}
                )
            )
            self.assertEqual(pins.GitHub(path, NOW, refresh=True)("/x"), {"ok": True})

    def test_network_failure_names_the_cause(self):
        def refuse(*_args, **_kwargs):
            raise OSError("no route")

        pins.fetch = refuse
        with self.assertRaises(pins.LookupFailed) as caught:
            pins.GitHub(None, NOW)("/repos/o/r/releases")
        self.assertIn("could not reach", str(caught.exception))


if __name__ == "__main__":
    unittest.main()
