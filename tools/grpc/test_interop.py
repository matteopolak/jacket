"""Public interop reports must not copy private diagnostics or machine identity."""
import argparse
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import interop


class PublicReports(unittest.TestCase):
    def test_python_identity_must_match_the_manifest_patch_version(self):
        versions = json.dumps({"python": "3.14.3", "grpcio": "1.78.0",
                               "grpcio-tools": "1.78.0", "protobuf": "6.33.5"})
        with patch.object(interop, "load_pins", return_value={"python_grpc": "3.14.4"}), \
                patch.object(interop.subprocess, "check_output", return_value=versions):
            with self.assertRaisesRegex(RuntimeError, "3.14.4"):
                interop.peer_metadata(Path("peer"), Path("compiler"))

    def test_peer_identity_keeps_versions_and_digests_only(self):
        with tempfile.TemporaryDirectory(prefix="private-user-host-") as folder:
            compiler = Path(folder) / "private-compiler"
            compiler.write_bytes(b"compiler fixture")
            versions = json.dumps({"python": "3.14.3 (private-user@private-host /private/worktree)",
                                   "grpcio": "1.78.0", "grpcio-tools": "1.78.0", "protobuf": "6.33.5",
                                   "unexpected_context": "private-user@private-host TOKEN=secret"})
            with patch.object(interop.subprocess, "check_output", side_effect=[versions, "libprotoc 31.1", "a" * 40, ""]), \
                    patch.object(interop.platform, "system", return_value="Linux"), \
                    patch.object(interop.platform, "machine", return_value="x86_64"), \
                    patch.object(interop.platform, "platform", side_effect=AssertionError("raw host context")):
                result = interop.peer_metadata(Path("private-peer"), compiler)
            text = json.dumps(result)
            self.assertEqual(result["python"], "3.14.3")
            self.assertEqual(result["platform"], {"os": "Linux", "architecture": "x86_64"})
            self.assertNotIn("private-", text)
            self.assertNotIn(folder, text)
            self.assertEqual(len(result["compiler_sha256"]), 64)

    def test_failure_report_excludes_exception_paths_and_environment(self):
        with tempfile.TemporaryDirectory() as folder:
            args = argparse.Namespace(output=Path(folder))
            with patch.object(interop, "matrix", side_effect=RuntimeError("private-user@private-host /private/worktree TOKEN=secret")):
                with self.assertRaises(RuntimeError):
                    interop.run_report(args)
            self.assertEqual(json.loads((args.output / "result.json").read_text()),
                             {"passed": False, "failure_kind": "RuntimeError"})

    def test_success_report_records_acceptance(self):
        with tempfile.TemporaryDirectory() as folder:
            args = argparse.Namespace(output=Path(folder))
            result = {"environment": {"python": "3.14.3"}, "results": [{"passed": True}]}
            with patch.object(interop, "matrix", return_value=result):
                interop.run_report(args)
            self.assertEqual(json.loads((args.output / "result.json").read_text()), dict(result, passed=True))


if __name__ == "__main__":
    unittest.main()
