import importlib.util
from datetime import datetime, timedelta, timezone
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "age", Path(__file__).with_name("check_dependency_age.py")
)
age = importlib.util.module_from_spec(spec)
spec.loader.exec_module(age)


class AgePolicyTests(unittest.TestCase):
    def test_transitive_and_existing_entries(self):
        now = datetime(2026, 10, 1, tzinfo=timezone.utc)
        source = "registry+https://github.com/rust-lang/crates.io-index"
        packages = [
            {"name": "old", "version": "1", "source": source},
            {"name": "young", "version": "1", "source": source},
            {"name": "internal", "version": "1"},
        ]

        def version(name, _):
            return {
                "created_at": (
                    now - timedelta(days=14 if name == "old" else 13)
                ).isoformat(),
                "yanked": False,
            }

        errors = age.check(packages, version, now)
        self.assertEqual(len(errors), 1)
        self.assertIn("young", errors[0])

    def test_unknown_sources_fail_closed(self):
        errors = age.check(
            [{"name": "x", "version": "1", "source": "git+https://example.com"}],
            lambda *_: self.fail(),
            datetime.now(timezone.utc),
        )
        self.assertIn("unverified", errors[0])

    def test_pinned_git_commits_are_aged(self):
        now = datetime(2026, 10, 1, tzinfo=timezone.utc)
        sha = "b7cbeed24af810e14cb36d703f057167b4c033a4"
        source = f"git+https://github.com/TheDan64/inkwell?rev={sha}#{sha}"
        seen = []

        def commit(owner, repo, rev):
            seen.append((owner, repo, rev))
            return (now - timedelta(days=days)).isoformat()

        days = 14
        self.assertEqual(
            age.check(
                [{"name": "inkwell", "version": "0.10.0", "source": source}],
                lambda *_: self.fail(),
                now,
                commit,
            ),
            [],
        )
        self.assertEqual(seen, [("TheDan64", "inkwell", sha)])
        days = 13
        errors = age.check(
            [{"name": "inkwell", "version": "0.10.0", "source": source}],
            lambda *_: self.fail(),
            now,
            commit,
        )
        self.assertIn("younger than 14 days", errors[0])

    def test_unpinned_git_sources_fail_closed(self):
        sha = "b7cbeed24af810e14cb36d703f057167b4c033a4"
        sources = [
            f"git+https://github.com/TheDan64/inkwell?branch=master#{sha}",
            f"git+https://github.com/TheDan64/inkwell#{sha}",
            f"git+https://github.com/TheDan64/inkwell?rev=b7cbeed#{sha}",
            f"git+https://github.com/TheDan64/inkwell?rev={'0' * 40}#{sha}",
            f"git+https://gitlab.com/TheDan64/inkwell?rev={sha}#{sha}",
        ]
        for source in sources:
            errors = age.check(
                [{"name": "x", "version": "1", "source": source}],
                lambda *_: self.fail(),
                datetime.now(timezone.utc),
                lambda *_: self.fail(),
            )
            self.assertIn("unverified", errors[0], source)

    def test_yanked(self):
        p = {
            "name": "x",
            "version": "1",
            "source": "registry+https://github.com/rust-lang/crates.io-index",
        }
        errors = age.check(
            [p],
            lambda *_: {"created_at": "2020-01-01T00:00:00Z", "yanked": True},
            datetime.now(timezone.utc),
        )
        self.assertIn("yanked", errors[0])

    def test_cache_entries_are_trusted_only_while_fresh_and_well_formed(self):
        now = datetime(2026, 10, 1, tzinfo=timezone.utc)
        entry = {
            "created_at": "2026-01-01T00:00:00+00:00",
            "yanked": False,
            "fetched": (now - timedelta(days=1)).isoformat(),
        }
        stale = entry | {"fetched": (now - timedelta(days=4)).isoformat()}
        future = entry | {"fetched": (now + timedelta(days=1)).isoformat()}
        self.assertTrue(age.fresh(entry, now))
        self.assertFalse(age.fresh(stale, now))
        self.assertFalse(age.fresh(future, now))
        self.assertFalse(age.fresh(entry | {"yanked": "no"}, now))
        self.assertFalse(age.fresh({}, now))

    def test_unreadable_cache_is_empty(self):
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "c.json"
            self.assertEqual(age.load_cache(path), {})
            path.write_text("not json")
            self.assertEqual(age.load_cache(path), {})
            path.write_text("[1]")
            self.assertEqual(age.load_cache(path), {})


if __name__ == "__main__":
    unittest.main()
