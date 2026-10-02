import importlib.util
from datetime import datetime, timedelta, timezone
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("age", Path(__file__).with_name("check_dependency_age.py"))
age = importlib.util.module_from_spec(spec)
spec.loader.exec_module(age)
graph_spec = importlib.util.spec_from_file_location("graph", Path(__file__).with_name("analyze_macos_calls.py"))
graph = importlib.util.module_from_spec(graph_spec)
graph_spec.loader.exec_module(graph)


class AgePolicyTests(unittest.TestCase):
    def test_transitive_and_existing_entries(self):
        now = datetime(2026, 10, 1, tzinfo=timezone.utc)
        source = "registry+https://github.com/rust-lang/crates.io-index"
        packages = [{"name": "old", "version": "1", "source": source}, {"name": "young", "version": "1", "source": source}, {"name": "internal", "version": "1"}]
        def version(name, _):
            return {"created_at": (now - timedelta(days=14 if name == "old" else 13)).isoformat(), "yanked": False}
        errors = age.check(packages, version, now)
        self.assertEqual(len(errors), 1)
        self.assertIn("young", errors[0])

    def test_unknown_sources_fail_closed(self):
        errors = age.check([{"name": "x", "version": "1", "source": "git+https://example.com"}], lambda *_: self.fail(), datetime.now(timezone.utc))
        self.assertIn("unverified", errors[0])

    def test_yanked(self):
        p = {"name": "x", "version": "1", "source": "registry+https://github.com/rust-lang/crates.io-index"}
        errors = age.check([p], lambda *_: {"created_at": "2020-01-01T00:00:00Z", "yanked": True}, datetime.now(timezone.utc))
        self.assertIn("yanked", errors[0])


class CallGraphTests(unittest.TestCase):
    def test_shortest_paths_with_cycles(self):
        calls = {"main": {"worker"}, "worker": {"main", "system"}}
        self.assertEqual(graph.shortest_path(calls, "main", "system"), ["main", "worker", "system"])
        self.assertIsNone(graph.shortest_path(calls, "main", "absent"))


if __name__ == "__main__":
    unittest.main()
