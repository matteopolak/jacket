"""Source-only controls; target exclusions never imply native verification."""

from pathlib import Path
import re
import tempfile
import unittest

from verification_policy import (
    private_library_root,
    module_target_condition,
    module_result,
)


class PrivateLayout(unittest.TestCase):
    def test_existing_cache_fixture_keeps_repository_relative_dependencies(self):
        from verify import PRIVATE_UNIT_DEPENDENCIES

        root = Path(__file__).resolve().parents[2]
        body = (root / "stdlib/rpmalloc/tests/no-thread-cache.jai").read_text()
        dependencies = PRIVATE_UNIT_DEPENDENCIES["rpmalloc-no-thread-cache"]
        library = Path("/owned/private-units/stdlib")
        for relative in re.findall(r'#load "([^"]+)"', body):
            source = (
                "stdlib/Bindings_Generator/tests/driven/sdk/"
                + relative.split("tests/driven/sdk/")[1]
            )
            actual = (library / "rpmalloc/tests" / relative).resolve()
            staged = (library / dependencies[source]).resolve()
            self.assertEqual(staged, actual)

    def test_actual_dxc_loads_resolve_inside_private_library(self):
        from verify import relocate_private_loads

        root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            library = private_library_root(Path(directory))
            for name, component in (
                ("dxc-wide", "dxc-wide"),
                ("dxc-compile-unit", "dxc-compile"),
            ):
                data = relocate_private_loads(
                    (root / "stdlib/Bindings_Generator/tests/driven/sdk" / (name + ".jai")).read_bytes()
                )
                relative = re.search(rb'#load "([^"]+)"', data)[1].decode()
                placed = library / "dxc_compiler" / (name + ".jai")
                resolved = (placed.parent / relative).resolve()
                expected = (library / "internal/bindings" / component / "module.jai").resolve()
                self.assertEqual(resolved, expected)


class Targets(unittest.TestCase):
    def test_appkit_has_narrower_availability_than_parent(self):
        self.assertEqual(module_target_condition("Objective_C/AppKit"), "OS == .MACOS")
        self.assertIn("OS == .IOS", module_target_condition("Objective_C"))

    def test_supported_backend_error_stays_failure(self):
        for module, os_name, diagnostic in (
            ("macOS", "macos", "macOS requires its documented native platform"),
            ("Objective_C/AppKit", "macos", "Objective_C/AppKit requires macOS"),
        ):
            self.assertEqual(
                module_result(
                    module, os_name, "x64", {"exit": 1, "stderr": diagnostic}
                ),
                "failed",
            )

    def test_unsupported_target_requires_actual_guard_diagnostic(self):
        for module, diagnostic in (
            ("macOS", "macOS requires its documented native platform"),
            ("Objective_C/AppKit", "Objective_C/AppKit requires macOS"),
        ):
            self.assertEqual(
                module_result(
                    module, "linux", "arm64", {"exit": 1, "stderr": diagnostic}
                ),
                "unsupported",
            )
            self.assertEqual(
                module_result(
                    module, "linux", "arm64", {"exit": 1, "stderr": "missing file"}
                ),
                "failed",
            )
            self.assertEqual(
                module_result(module, "linux", "arm64", {"exit": 0, "stderr": ""}),
                "failed",
            )


if __name__ == "__main__":
    unittest.main()
