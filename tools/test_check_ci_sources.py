import tempfile
import unittest
from pathlib import Path

from check_ci_sources import inspect_includes


class PublicIncludes(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.source = self.root / "crates/example/src/lib.rs"
        self.source.parent.mkdir(parents=True)

    def inspect(self, include, relative, public=True):
        self.source.write_text(f'const SOURCE: &str = include_str!("{include}");\n')
        target = self.root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("self-authored inert fixture")
        files = [self.source, target] if public else [self.source]
        return inspect_includes(self.root, files)

    def test_public_fixture_is_available_in_a_clean_checkout(self):
        self.assertEqual(self.inspect("../fixtures/source.jai", "crates/example/fixtures/source.jai"), (1, []))

    def test_existing_unpublished_input_does_not_establish_ci_availability(self):
        count, errors = self.inspect("../fixtures/source.jai", "crates/example/fixtures/source.jai", public=False)
        self.assertEqual(count, 1)
        self.assertIn("absent from public checkout", errors[0])

    def test_authored_prelude_is_available_without_vendor_exception(self):
        self.assertEqual(self.inspect("../../../stdlib/prelude/module.jai", "stdlib/prelude/module.jai"), (1, []))

    def test_vendor_inputs_are_excluded_without_exceptions(self):
        for path in ("vendor/compiler/modules/module.jai", "vendor/other/module.jai"):
            _, errors = self.inspect("../../../" + path, path)
            self.assertIn("excluded input", errors[0])

    def test_include_symlink_cannot_escape_checkout(self):
        self.source.write_text('include_bytes!("payload");')
        payload = self.source.parent / "payload"
        payload.symlink_to(Path(__file__).resolve())
        _, errors = inspect_includes(self.root, [self.source, payload])
        self.assertIn("escapes", errors[0])


if __name__ == "__main__":
    unittest.main()
