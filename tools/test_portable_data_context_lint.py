"""Public import ownership and complete private lint coverage regressions."""

import ast
import unittest

import portable_data_context_lint as lint


class PortableDataContexts(unittest.TestCase):
    def test_public_context_never_imports_internal_modules(self):
        source = lint.public_context_source()
        self.assertNotIn("internal/", source)
        for name in (
            "Ico_File",
            "Icon",
            "Simple_Package",
            "Subtitles",
            "Windows_Resources",
        ):
            self.assertIn(f'#import "{name}"', source)

    def test_every_owned_source_has_an_applicable_target_context(self):
        roots = []
        tree = ast.parse((lint.ROOT / "tools/portable_data/run.py").read_text())
        for node in tree.body:
            if isinstance(node, ast.Assign) and any(
                isinstance(target, ast.Name)
                and target.id in {"sources", "windows_sources"}
                for target in node.targets
            ):
                roots.extend(ast.literal_eval(node.value))
        owned = lint.owned_files([lint.ROOT / path for path in roots])
        required = set()
        for target_os, cpu in lint.PROFILES:
            library, fixtures = lint.profile_sources(owned, target_os)
            required.update(library + fixtures)
        self.assertEqual(required, set(owned))
        library, fixtures = lint.profile_sources(owned, "windows")
        for name in (
            "resource-wire/manifest.jai",
            "resource-wire/tool-versions.jai",
            "simple-package/module.jai",
            "subtitles/module.jai",
        ):
            self.assertIn(lint.ROOT / "stdlib/internal" / name, library)
        self.assertIn(
            lint.ROOT / "stdlib/Windows_Resources/tests/discovery-windows.jai", fixtures
        )

    def test_failed_or_unreported_bodies_do_not_establish_coverage(self):
        path = lint.ROOT / "stdlib/internal/resource-wire/manifest.jai"
        report = {"contexts": [{"complete": False, "visited_files": [str(path)]}]}
        self.assertFalse(lint.complete_checked_paths(report, {path}))
        report["contexts"][0]["complete"] = True
        self.assertFalse(lint.complete_checked_paths(report, set()))
        self.assertEqual(lint.complete_checked_paths(report, {path}), {path})

    def test_missing_private_body_is_a_required_failure(self):
        path = lint.ROOT / "stdlib/internal/resource-wire/tool-versions.jai"
        with self.assertRaisesRegex(RuntimeError, "tool-versions"):
            lint.require_coverage({path}, set())


if __name__ == "__main__":
    unittest.main()
