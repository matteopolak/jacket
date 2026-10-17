"""Fixture ownership and exception-safe private placement, without native claims."""

from contextlib import ExitStack
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

NATIVE_TOOLS = Path(__file__).resolve().parent / "native_bindings"
sys.path.insert(0, str(NATIVE_TOOLS))
SPEC = importlib.util.spec_from_file_location(
    "native_verifier", NATIVE_TOOLS / "verify.py"
)
verifier = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(verifier)


class NativeVerifierInputs(unittest.TestCase):
    def test_module_owned_fixture_wins_over_stale_external_copy(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            owned = root / "stdlib/SDL/tests/sdl-layout.jai"
            owned.parent.mkdir(parents=True)
            owned.write_text("original owned fixture")
            stale = root / "stdlib/Bindings_Generator/tests/driven/sdk/sdl-layout.jai"
            stale.parent.mkdir(parents=True)
            stale.write_text("stale external copy")
            self.assertEqual(
                verifier.fixture_source(root, "sdl-layout.jai"), owned.resolve()
            )
            self.assertEqual(
                verifier.fixture_source(root, "abi.jai"),
                root / "stdlib/Bindings_Generator/tests/driven/sdk/abi.jai",
            )
            owned.unlink()
            with self.assertRaises(FileNotFoundError):
                verifier.fixture_source(root, "sdl-layout.jai")

    def test_private_placement_restores_existing_source_on_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "original.jai"
            source.write_bytes(b"original fixture\n")
            target = root / "stdlib/Example/unit.jai"
            target.parent.mkdir(parents=True)
            target.write_bytes(b"existing independent source\n")
            target.chmod(0o640)
            original_mode = target.stat().st_mode & 0o777
            original_mtime = target.stat().st_mtime_ns
            with patch.dict(verifier.PRIVATE_UNITS, {"test": "Example/unit.jai"}):
                with self.assertRaisesRegex(RuntimeError, "compile failed"):
                    with ExitStack() as cleanup:
                        verifier.place_private_unit(
                            "test", source, root / "stdlib", {}, cleanup
                        )
                        self.assertEqual(target.read_bytes(), source.read_bytes())
                        raise RuntimeError("compile failed")
            self.assertEqual(target.read_bytes(), b"existing independent source\n")
            self.assertEqual(target.stat().st_mode & 0o777, original_mode)
            self.assertEqual(target.stat().st_mtime_ns, original_mtime)

    def test_private_placement_removes_new_dependency_directories(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "original.jai"
            source.write_bytes(b"original fixture\n")
            stdlib = root / "stdlib"
            stdlib.mkdir()
            with (
                patch.object(verifier, "ROOT", root),
                patch.dict(verifier.PRIVATE_UNITS, {"test": "Example/unit.jai"}),
                patch.dict(
                    verifier.PRIVATE_UNIT_DEPENDENCIES,
                    {"test": {"original.jai": "Example/generated/layout.jai"}},
                ),
            ):
                with ExitStack() as cleanup:
                    verifier.place_private_unit("test", source, stdlib, {}, cleanup)
                    self.assertTrue((stdlib / "Example/generated/layout.jai").is_file())
            self.assertEqual(list(stdlib.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
