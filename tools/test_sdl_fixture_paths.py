"""SDL fixture migration/staging checks; no compiler, SDK or native execution."""

import hashlib
import ast
import importlib.util
from sdl_test_support import source_path
from pathlib import Path
import sys
from sdl_test_support import temporary_directory
import unittest
from unittest.mock import patch


class SdlFixturePaths(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        helper = Path(
            source_path(
                "SDL_FIXTURE_TEST_HELPER", "tools/native_bindings/fixture_paths.py"
            )
        )
        spec = importlib.util.spec_from_file_location("fixture_paths", helper)
        cls.paths = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = cls.paths
        spec.loader.exec_module(cls.paths)
        deps = Path(
            source_path("SDL_FIXTURE_TEST_DEPENDENCIES", "tools/native_bindings")
        )
        sys.path.insert(0, str(deps))
        spec = importlib.util.spec_from_file_location(
            "sdl_fixture_verify",
            source_path("SDL_FIXTURE_TEST_VERIFY", "tools/native_bindings/verify.py"),
        )
        cls.verify = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.verify)
        cls.verify_source = Path(
            source_path("SDL_FIXTURE_TEST_VERIFY", "tools/native_bindings/verify.py")
        ).read_text()
        cls.shared = Path(source_path("SDL_FIXTURE_TEST_ROOT", "."))

    def setUp(self):
        self.temporary = temporary_directory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_all_three_sdl_fixtures_resolve_only_to_canonical_module_tests(self):
        for name in ["sdl-layout.jai", "sdl-rwops.jai", "sdl-wm.jai"]:
            expected = self.root / "stdlib/SDL/tests" / name
            expected.parent.mkdir(parents=True, exist_ok=True)
            expected.write_text("independent non-executable path-selection fixture\n")
            self.assertEqual(
                self.paths.fixture_source(self.root, name), expected.resolve()
            )

    def test_missing_canonical_fixture_is_fatal_even_if_old_path_exists(self):
        old = self.root / "tests/native-bindings/sdl-layout.jai"
        old.parent.mkdir(parents=True)
        old.write_text("stale path fixture, never an ABI oracle\n")
        with self.assertRaises(FileNotFoundError):
            self.paths.fixture_source(self.root, "sdl-layout.jai")

    def test_unmoved_fixtures_keep_their_existing_location(self):
        self.assertEqual(
            self.paths.fixture_source(self.root, "sdl.jai"),
            self.root / "tests/native-bindings/sdl.jai",
        )
        self.assertEqual(
            self.paths.fixture_source(self.root, "posix-abi.jai"),
            self.root / "tests/native-bindings/posix-abi.jai",
        )

    def test_source_and_runtime_loops_both_use_the_migration_inventory(self):
        tree = ast.parse(self.verify_source)
        calls = [
            node
            for node in ast.walk(tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "fixture_source"
        ]
        self.assertEqual(len(calls), 2)
        self.assertTrue(
            any(
                isinstance(node.args[1], ast.Name) and node.args[1].id == "filename"
                for node in calls
            )
        )
        self.assertTrue(any(isinstance(node.args[1], ast.JoinedStr) for node in calls))
        self.assertIn("sdl-layout", self.verify.PRIVATE_UNITS)
        self.assertIn("sdl-rwops", self.verify.PRIVATE_UNITS)
        self.assertIn("sdl-wm", self.verify.PRIVATE_UNITS)

    def test_real_canonical_layout_and_payload_assertions_are_not_replaced(self):
        source = self.paths.fixture_source(self.shared, "sdl-layout.jai").read_text()
        for assertion in [
            "size_matches(0",
            "size_matches(6",
            "native_offset(4)",
            "rw_offset(cast(u32) index)",
            "user_offset(cast(u32) index)",
            "native_alignment(0)",
            "native_alignment(1)",
            "fill_user(*guarded.event",
            "for byte: guarded.before assert",
            "for byte: guarded.after assert",
        ]:
            self.assertIn(assertion, source)

    def test_module_local_sources_stage_unchanged_inside_real_private_boundary(self):
        staged = self.root / "private-stdlib"
        hashes = {}
        with patch.object(self.verify, "ROOT", self.shared):
            for filename in self.paths.MODULE_LOCAL_FIXTURES:
                name = Path(filename).stem
                source = self.paths.fixture_source(self.shared, filename)
                self.assertEqual(
                    self.verify.PRIVATE_UNITS[name], "SDL/tests/" + filename
                )
                placed = self.verify.place_private_unit(name, source, staged, hashes)
                self.assertEqual(placed, staged / "SDL/tests" / filename)
                self.assertEqual(placed.read_bytes(), source.read_bytes())
                self.assertEqual(
                    hashes["SDL/tests/" + filename],
                    hashlib.sha256(source.read_bytes()).hexdigest(),
                )


if __name__ == "__main__":
    unittest.main()
