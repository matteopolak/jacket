"""Source/configuration checks only; actual Windows SDK execution is required."""

from pathlib import Path
import unittest


class ProviderWorkflow(unittest.TestCase):
    def test_both_cpu_probes_precede_required_webgpu(self):
        text = Path(".github/workflows/windows-native.yml").read_text()
        for triple, arch in [
            ("x86_64-pc-windows-msvc", "amd64"),
            ("aarch64-pc-windows-msvc", "arm64"),
        ]:
            probe = text.index(
                "--jaic target/"
                + triple
                + "/debug/jaic.exe --stdlib stdlib --import-library"
            )
            gpu = text.index("tools/test_webgpu_native.py --jaic target/" + triple)
            self.assertLess(probe, gpu)
            self.assertIn("-Arch " + arch + " -HostArch amd64", text[:probe])
        self.assertEqual(
            text.count("Selected SDK LIB search has no d3dcompiler.lib"), 2
        )
        self.assertEqual(
            text.count("Actual D3DCompiler provider interpreter/native check failed"), 2
        )

    def test_sdk_selection_is_not_dll_renaming_or_a_capability_skip(self):
        text = Path(".github/workflows/windows-native.yml").read_text()
        self.assertEqual(text.count("$env:LIB.Split"), 2)
        self.assertEqual(text.count("Select-Object -First 1"), 2)
        for line in text.splitlines():
            if "tools/test_webgpu_native.py --jaic" in line:
                self.assertIn("--require-adapter", line)
                if "--test window" in line:
                    self.assertIn("--frames 120", line)


if __name__ == "__main__":
    unittest.main()
