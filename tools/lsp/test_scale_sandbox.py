"""Required upstream scale cases never fall back to an unconfined subprocess."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lsp import scale_sandbox


class ScaleSandboxTests(unittest.TestCase):
    def test_missing_sandbox_refuses_even_in_ci(self):
        with patch.object(scale_sandbox.Sandbox, "available", return_value="unavailable"):
            with self.assertRaisesRegex(RuntimeError, "unavailable"):
                scale_sandbox.required_sandbox([], [])

    def test_private_worker_and_canonical_inputs_are_handed_off_once(self):
        with tempfile.TemporaryDirectory() as temporary:
            scratch = Path(temporary)
            command, env = scale_sandbox.child_command(Path("/build/lsp_scale_worker"), scratch, Path("/source/main.jai"), Path("/library"))
            self.assertEqual(command, ["/build/lsp_scale_worker", str(Path("/source/main.jai")), str(Path("/library"))])
            self.assertNotIn("JAILSP_SCALE_SANDBOX_INPUTS", env)
            self.assertTrue(env["TMPDIR"].startswith(str(scratch)))

    def test_read_roots_include_ancestor_configs_but_not_their_directories(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "repo" / "corpus" / "upstream" / "focus" / "src" / "main.jai"
            source.parent.mkdir(parents=True)
            config = root / "repo" / "jacket.toml"
            config.write_text("[build]\n")
            reads = scale_sandbox.read_roots(root / "target" / "debug" / "deps" / "scale", root / "repo" / "stdlib", source)
            self.assertIn(config, reads)
            self.assertNotIn(config.parent, reads)
            self.assertIn(source.parents[1], reads)
