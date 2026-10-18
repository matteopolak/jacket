"""Real SDK cross-build tests: no Windows runtime or host SDK execution proof."""

import hashlib
from pathlib import Path
import struct
import tempfile
import unittest

import build_windows_resource_fixture as fixture


class ResourceFixture(unittest.TestCase):
    def test_real_five_frame_dib_pixels_and_mask(self):
        icon = fixture.make_ico()
        self.assertEqual(struct.unpack_from("<HHH", icon), (0, 1, 5))
        end = 86
        for index, size in enumerate((256, 128, 64, 32, 16)):
            width, height, colors, reserved, planes, bits, length, offset = (
                struct.unpack_from("<BBBBHHII", icon, 6 + index * 16)
            )
            self.assertEqual(
                (width or 256, height or 256, colors, reserved, planes, bits),
                (size, size, 0, 0, 1, 32),
            )
            self.assertEqual(offset, end)
            payload = icon[offset : offset + length]
            self.assertEqual(
                struct.unpack_from("<IiiHHI", payload), (40, size, size * 2, 1, 32, 0)
            )
            pixels = payload[40 : 40 + size * size * 4]
            self.assertEqual(len(pixels), size * size * 4)
            self.assertEqual(set(pixels[3::4]), {255})
            self.assertGreater(len(set(pixels)), 16)
            mask = payload[40 + len(pixels) :]
            self.assertEqual(len(mask), ((size + 31) // 32) * 4 * size)
            self.assertFalse(any(mask))
            end += length
        self.assertEqual(len(icon), end)

    def test_actual_sdk_both_machines_and_deterministic_payload(self):
        with tempfile.TemporaryDirectory() as directory:
            receipts = []
            for label, cpu in (
                ("first-x64", "x64"),
                ("arm64", "arm64"),
                ("second-x64", "x64"),
            ):
                output = Path(directory) / label
                receipt = fixture.build_fixture(output, cpu)
                receipts.append(receipt)
                self.assertEqual(receipt["status"], "verified_cross_build")
                self.assertFalse(receipt["native_verified"])
                self.assertEqual(
                    receipt["pe"]["machine"], {"x64": 0x8664, "arm64": 0xAA64}[cpu]
                )
                self.assertEqual(receipt["pe"]["entry_point"], 0)
                self.assertEqual(receipt["pe"]["resource_count"], 6)
                self.assertEqual(receipt["pe"]["group_bytes"], 76)
                for name, identity in receipt["artifacts"].items():
                    self.assertEqual(
                        hashlib.sha256((output / name).read_bytes()).hexdigest(),
                        identity["sha256"],
                    )
                icon = (output / "resource.ico").read_bytes()
                pe = (output / "resource-only.dll").read_bytes()
                with self.assertRaisesRegex(ValueError, "machine"):
                    fixture.inspect_pe(pe, "arm64" if cpu == "x64" else "x64", icon)
                pe_offset = struct.unpack_from("<I", pe, 0x3C)[0]
                damaged = bytearray(pe)
                struct.pack_into("<I", damaged, pe_offset + 24 + 16, 1)
                with self.assertRaisesRegex(ValueError, "entry"):
                    fixture.inspect_pe(damaged, cpu, icon)
                damaged = bytearray(pe)
                # Corrupt an actual image payload, not padding in the PE file.
                payload = icon[86 : 86 + 128]
                location = pe.find(payload)
                self.assertGreaterEqual(location, 0)
                damaged[location + 47] ^= 1
                with self.assertRaisesRegex(ValueError, "payload"):
                    fixture.inspect_pe(damaged, cpu, icon)
            for name in ("resource.ico", "resource-only.dll", "resource.res"):
                self.assertEqual(
                    receipts[0]["artifacts"][name]["sha256"],
                    receipts[2]["artifacts"][name]["sha256"],
                )
            self.assertEqual(
                receipts[0]["artifacts"]["resource.ico"],
                receipts[1]["artifacts"]["resource.ico"],
            )
            self.assertNotEqual(
                receipts[0]["artifacts"]["resource-only.dll"],
                receipts[1]["artifacts"]["resource-only.dll"],
            )

    def test_truncated_or_non_pe_input_is_rejected(self):
        for data in (b"", b"MZ", b"MZ" + bytes(100)):
            with self.assertRaises(ValueError):
                fixture.inspect_pe(data, "x64", fixture.make_ico())

    def test_missing_sdk_is_fatal_and_never_publishes_dll(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "missing-sdk"
            with self.assertRaisesRegex((ValueError, FileNotFoundError), "tool"):
                fixture.build_fixture(
                    output, "x64", rc=str(Path(directory) / "no-resource-tool")
                )
            self.assertFalse((output / "resource-only.dll").exists())

    def test_caller_fixture_is_not_silently_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            owned = output / "resource.ico"
            owned.write_bytes(b"caller-owned data")
            with self.assertRaisesRegex(ValueError, "fresh"):
                fixture.build_fixture(output, "x64")
            self.assertEqual(owned.read_bytes(), b"caller-owned data")


if __name__ == "__main__":
    unittest.main()
