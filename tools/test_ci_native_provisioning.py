"""Configuration checks only: no apt, SDK, compiler or native program execution."""

from pathlib import Path
import re
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class NativeProvisioning(unittest.TestCase):
    def packages(self):
        result = subprocess.run(
            ["bash", str(ROOT / "tools/install_ci_native_linux.sh"), "--list"],
            capture_output=True,
            text=True,
            check=True,
            timeout=5,
        )
        return result.stdout.splitlines()

    def test_display_audio_and_xtest_backends_have_development_inputs(self):
        required = {
            "libx11-dev",
            "libxtst-dev",
            "libxrandr-dev",
            "libxcursor-dev",
            "libxfixes-dev",
            "libxi-dev",
            "libxss-dev",
            "libwayland-dev",
            "libxkbcommon-dev",
            "libdecor-0-dev",
            "libasound2-dev",
            "libpulse-dev",
            "libpipewire-0.3-dev",
            "libdrm-dev",
            "libgbm-dev",
        }
        self.assertLessEqual(required, set(self.packages()))

    def test_package_list_is_unique_and_keeps_existing_native_consumers(self):
        packages = self.packages()
        self.assertEqual(len(packages), len(set(packages)))
        self.assertLessEqual(
            {
                "libsdl2-dev",
                "libfreetype-dev",
                "mesa-vulkan-drivers",
                "xvfb",
                "xauth",
                "xclip",
                "x11-apps",
                "libfontconfig1-dev",
                "libc++-dev",
            },
            set(packages),
        )

    def test_invalid_arguments_fail_before_provisioning(self):
        result = subprocess.run(
            ["bash", str(ROOT / "tools/install_ci_native_linux.sh"), "--disable-x11"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr)

    def test_shell_syntax_and_backend_policy(self):
        path = ROOT / "tools/install_ci_native_linux.sh"
        subprocess.run(["bash", "-n", str(path)], check=True, timeout=5)
        source = path.read_text()
        self.assertIn("set -euo pipefail", source)
        self.assertIn("sudo apt-get update", source)
        self.assertNotIn("SDL_", source)
        self.assertNotIn("|| true", source)


if __name__ == "__main__":
    unittest.main()
