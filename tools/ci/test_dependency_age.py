import importlib.util
from datetime import datetime, timedelta, timezone
from pathlib import Path
import tempfile
import unittest

from toolkit.age_policy import load_cache, save_cache

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

    def test_index_paths_follow_the_sparse_layout(self):
        self.assertEqual(age.index_path("a"), "1/a")
        self.assertEqual(age.index_path("cc"), "2/cc")
        self.assertEqual(age.index_path("Log"), "3/l/log")
        self.assertEqual(age.index_path("anstream"), "an/st/anstream")

    def test_index_versions_carry_publish_time_and_yank_state(self):
        text = "\n".join(
            [
                '{"name":"x","vers":"1.0.0","pubtime":"2020-01-01T00:00:00Z","yanked":false}',
                '{"name":"x","vers":"1.0.1","pubtime":"2020-02-01T00:00:00Z","yanked":true}',
                '{"name":"x","vers":"0.9.0","yanked":false}',
                "",
            ]
        )
        found = age.versions(text)
        self.assertEqual(found["1.0.0"], {"created_at": "2020-01-01T00:00:00Z", "yanked": False})
        self.assertTrue(found["1.0.1"]["yanked"])
        self.assertNotIn("0.9.0", found)

    def test_versions_missing_from_the_index_fail_closed(self):
        p = {
            "name": "x",
            "version": "1",
            "source": "registry+https://github.com/rust-lang/crates.io-index",
        }
        errors = age.check([p], lambda *_: None, datetime.now(timezone.utc))
        self.assertIn("not in the crates.io index", errors[0])

    def test_saving_the_cache_leaves_no_temporary_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "c.json"
            save_cache(path, {"a": 1})
            save_cache(path, {"b": 2})
            self.assertEqual(load_cache(path), {"b": 2})
            self.assertEqual([p.name for p in Path(tmp).iterdir()], ["c.json"])

    def test_unreadable_cache_is_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "c.json"
            self.assertEqual(load_cache(path), {})
            path.write_text("not json")
            self.assertEqual(load_cache(path), {})
            path.write_text("[1]")
            self.assertEqual(load_cache(path), {})


if __name__ == "__main__":
    unittest.main()
