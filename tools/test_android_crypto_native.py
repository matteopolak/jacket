"""Contract tests for the configured canonical crypto packaging workflow."""

import json
from pathlib import Path
import subprocess
import sys
import unittest

from portable_toolchain import assigned_target
from openssl_sdk import discover_sdk

ROOT = Path(__file__).resolve().parents[1]


class CryptoPackage(unittest.TestCase):
    def test_real_sdk_package(self):
        output = assigned_target() / "android-crypto-package-tests/stdlib"
        result = subprocess.run(
            [
                sys.executable,
                "-B",
                str(ROOT / "tools/build_android_crypto.py"),
                "--openssl-prefix",
                str(discover_sdk().prefix),
                "--stdlib",
                str(output),
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        package = output / "internal/android-toolchain/android-crypto"
        for name in [
            "libcrypto.a",
            "libcrypto.dylib",
            "licenses/openssl/LICENSE.txt",
            "licenses/openssl/AUTHORS.md",
            "provenance.json",
        ]:
            self.assertTrue((package / name).is_file(), name)
        provenance = json.loads((package / "provenance.json").read_text())
        self.assertEqual(provenance["shared_identity"], "@rpath/libcrypto.dylib")
        self.assertEqual(provenance["third_party_runtime_dependencies"], [])
        self.assertEqual(provenance["openssl_major"], 3)
        self.assertIn("libcrypto.a", provenance["sha256"])
        self.assertIn("libcrypto.dylib", provenance["sha256"])
        self.assertEqual(
            set(provenance["sha256"]),
            {
                "libcrypto.a",
                "libcrypto.dylib",
                "licenses/openssl/LICENSE.txt",
                "licenses/openssl/AUTHORS.md",
            },
        )
        self.assertEqual(
            set(provenance["runtime_versions"].values()), {provenance["implementation"]}
        )

    def test_missing_sdk_rejected(self):
        output = assigned_target() / "android-crypto-package-tests/missing"
        result = subprocess.run(
            [
                sys.executable,
                "-B",
                str(ROOT / "tools/build_android_crypto.py"),
                "--openssl-prefix",
                str(output / "absent"),
                "--stdlib",
                str(output),
            ],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("OpenSSL SDK", result.stderr)

    def test_wrong_major_rejected_before_linking(self):
        output = assigned_target() / "android-crypto-package-tests/wrong-major"
        sdk = output / "sdk"
        manifest = json.loads((ROOT / "tools/android-crypto-native.json").read_text())
        for name in manifest["sdk_inputs"]:
            path = sdk / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"rejection fixture; never linked")
        (sdk / "include/openssl/opensslv.h").write_text(
            "#define OPENSSL_VERSION_MAJOR 4\n"
        )
        result = subprocess.run(
            [
                sys.executable,
                "-B",
                str(ROOT / "tools/build_android_crypto.py"),
                "--openssl-prefix",
                str(sdk),
                "--stdlib",
                str(output / "stdlib"),
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("OPENSSL_VERSION_MAJOR 3", result.stderr)
        self.assertFalse(
            (output / "stdlib/internal/android-toolchain/android-crypto").exists()
        )


if __name__ == "__main__":
    unittest.main()
