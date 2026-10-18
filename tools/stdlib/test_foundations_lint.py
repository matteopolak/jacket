"""Public-context planning must retain complete owned source accounting."""

import unittest

import check_foundations as foundations


class FoundationsLintTests(unittest.TestCase):
    def test_private_fragments_are_requested_through_public_context(self):
        library, fixtures = foundations.lint_profile_sources("macos", "arm64")
        self.assertIn(
            foundations.ROOT / "stdlib/internal/files/native-unix.jai", library
        )
        self.assertIn(
            foundations.ROOT / "stdlib/internal/files/text-handler.jai", library
        )
        self.assertTrue(
            all(path.is_relative_to(foundations.ROOT / "stdlib") for path in library)
        )
        self.assertFalse(
            any("tests" not in path.parts for path in fixtures)
        )

    def test_x64_diagnostics_retain_an_actual_supported_lint_profile(self):
        division = (
            foundations.ROOT / "stdlib/Basic/tests/division-exercise.jai"
        )
        self.assertNotIn(
            division, foundations.lint_profile_sources("macos", "arm64")[1]
        )
        self.assertIn(division, foundations.lint_profile_sources("linux", "x64")[1])

    def test_every_positive_owned_source_is_required_somewhere(self):
        checked = set()
        for target_os, cpu in foundations.LINT_PROFILES:
            library, fixtures = foundations.lint_profile_sources(target_os, cpu)
            checked.update(library + fixtures)
        expected = (
            set(foundations.owned_sources())
            - foundations.negative_lint_sources().keys()
        )
        self.assertFalse(expected - checked)

    def test_corrected_configuration_and_context_fixtures_are_not_hidden(self):
        for target_os, cpu in foundations.LINT_PROFILES:
            fixtures = foundations.lint_profile_sources(target_os, cpu)[1]
            for name in (
                "Basic/tests/configuration",
                "Memory/tests/debugger-interface",
                "Basic/tests/format-grammar",
                "Basic/tests/format-aggregates",
                "Basic/tests/configuration-disabled",
                "Basic/tests/configuration-isolated-combined",
                "Memory/tests/debugger-interface-disabled",
            ):
                self.assertIn(foundations.ROOT / f"stdlib/{name}.jai", fixtures)


if __name__ == "__main__":
    unittest.main()
