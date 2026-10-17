"""Contract tests for the configured canonical crypto packaging workflow."""

from dataclasses import asdict
import hashlib
import json
import platform
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
        sdk = discover_sdk()
        descriptor = output.parent / "openssl-sdk.json"
        descriptor.parent.mkdir(parents=True, exist_ok=True)
        descriptor.write_text(json.dumps(asdict(sdk), default=str))
        result = subprocess.run(
            [
                sys.executable,
                "-B",
                str(ROOT / "tools/build_android_crypto.py"),
                "--openssl-sdk-descriptor",
                str(descriptor),
                "--stdlib",
                str(output),
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        package = output / "internal/android-toolchain/android-crypto"
        linux = platform.system() == "Linux"
        shared_name = "libcrypto.so" if linux else "libcrypto.dylib"
        provenance = json.loads((package / "provenance.json").read_text())
        notices = (
            {record["path"] for record in provenance["license_inputs"]}
            if linux
            else {"licenses/openssl/LICENSE.txt", "licenses/openssl/AUTHORS.md"}
        )
        expected_files = {"libcrypto.a", shared_name, *notices}
        if linux:
            expected_files.add(provenance["shared_identity"])
            self.assertTrue(notices, "actual SDK/distribution license notices required")
            self.assertTrue(
                all(name.startswith("licenses/openssl/") for name in notices)
            )
            self.assertEqual(
                (package / shared_name).resolve().name, provenance["shared_identity"]
            )
            self.assertEqual(provenance["sdk_descriptor"]["library"], str(sdk.library))
            self.assertTrue(provenance["linux_native_verified"])
            self.assertTrue(provenance["runtime_origin_verified"])
            self.assertEqual(
                provenance["sdk_archive_sha256"],
                hashlib.sha256(
                    (sdk.library.parent / "libcrypto.a").read_bytes()
                ).hexdigest(),
            )
            self.assertEqual(
                provenance["sdk_shared_sha256"],
                hashlib.sha256(sdk.library.read_bytes()).hexdigest(),
            )
            self.assertEqual(
                provenance["sha256"]["libcrypto.a"], provenance["sdk_archive_sha256"]
            )
            self.assertEqual(
                provenance["sha256"][shared_name], provenance["sdk_shared_sha256"]
            )

            self.assertTrue(provenance["sdk_public_headers"])
            self.assertTrue(provenance["runtime_dependency_provenance"])
            for record in provenance["license_inputs"]:
                self.assertEqual(
                    hashlib.sha256(Path(record["source"]).read_bytes()).hexdigest(),
                    record["sha256"],
                )
            for record in (
                provenance["sdk_public_headers"]
                + provenance["runtime_dependency_provenance"]
            ):
                self.assertEqual(
                    hashlib.sha256(Path(record["path"]).read_bytes()).hexdigest(),
                    record["sha256"],
                )
        for name in expected_files | {"provenance.json"}:
            self.assertTrue((package / name).is_file(), name)
        self.assertEqual(
            provenance["shared_identity"],
            "libcrypto.so.3" if linux else "@rpath/libcrypto.dylib",
        )
        self.assertEqual(provenance["third_party_runtime_dependencies"], [])
        self.assertEqual(provenance["openssl_major"], 3)
        self.assertIn("libcrypto.a", provenance["sha256"])
        self.assertIn(shared_name, provenance["sha256"])
        self.assertEqual(
            set(provenance["sha256"]),
            expected_files,
        )
        for name, digest in provenance["sha256"].items():
            self.assertEqual(
                hashlib.sha256((package / name).read_bytes()).hexdigest(), digest
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
