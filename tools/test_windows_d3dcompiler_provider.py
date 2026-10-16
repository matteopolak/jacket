"""Metadata/error-boundary tests, not simulated native shader compilation."""

import struct
import unittest

from check_windows_d3dcompiler_provider import import_library_dlls, object_dlls


def archive(*members):
    result = b"!<arch>\n"
    for name, data in members:
        header = name.ljust(16) + b"0".ljust(12) + b"0".ljust(6) + b"0".ljust(6)
        header += b"100644".ljust(8) + str(len(data)).encode().ljust(10) + b"`\n"
        result += header + data + (b"\n" if len(data) % 2 else b"")
    return result


def short(dll=b"independent_12.dll", machine=0x8664):
    payload = b"IndependentSymbol\0" + dll + b"\0"
    return (
        struct.pack("<HHHHIIHH", 0, 0xFFFF, 0, machine, 0, len(payload), 0, 0) + payload
    )


def coff(payload):
    return (
        struct.pack("<HHIIIHH", 0x8664, 1, 0, 0, 0, 0, 0)
        + struct.pack(
            "<8sIIIIIIHHI", b".idata$7", 0, 0, len(payload), 60, 0, 0, 0, 0, 0
        )
        + payload
    )


class ImportMetadata(unittest.TestCase):
    def test_short_import_x64_and_arm64(self):
        for machine in (0x8664, 0xAA64):
            self.assertEqual(
                object_dlls(short(machine=machine)), {"independent_12.dll"}
            )

    def test_gnu_descriptor_and_zero_reference(self):
        self.assertEqual(
            object_dlls(coff(b"independent_12.dll\0\0")), {"independent_12.dll"}
        )
        self.assertEqual(object_dlls(coff(b"\0\0\0\0")), set())
        self.assertEqual(object_dlls(coff(b"\x04\0\0\0")), set())

    def test_archive_boundaries_and_linker_members(self):
        data = archive(
            (b"/", b"unrelated_77.dll"),
            (b"item.obj/", short()),
            (b"second.obj/", short(b"another_9.dll")),
        )
        self.assertEqual(
            import_library_dlls(data), ["another_9.dll", "independent_12.dll"]
        )

    def test_arbitrary_strings_are_not_provider_metadata(self):
        self.assertEqual(
            import_library_dlls(archive((b"text/", b"unrelated_77.dll\0"))), []
        )

    def test_truncated_archive_member_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "truncated archive member"):
            import_library_dlls(archive((b"item.obj/", short()))[:-3])

    def test_truncated_short_record_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "truncated short import data"):
            object_dlls(short()[:-1])

    def test_paths_are_not_dll_basenames(self):
        with self.assertRaisesRegex(ValueError, "invalid import DLL basename"):
            object_dlls(short(b"../independent.dll"))

    def test_truncated_section_table_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "truncated COFF section table"):
            object_dlls(coff(b"independent_12.dll\0")[:30])

    def test_truncated_section_data_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "truncated COFF import section"):
            object_dlls(coff(b"independent_12.dll\0")[:-2])

    def test_non_import_bigobj_is_not_short_import(self):
        data = bytearray(short())
        struct.pack_into("<H", data, 4, 2)
        self.assertEqual(object_dlls(data), set())


if __name__ == "__main__":
    unittest.main()
