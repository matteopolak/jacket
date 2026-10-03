#!/usr/bin/env python3
"""Boundary checks for source-only platform API normalization."""
import unittest
from pathlib import Path

from rewrite_platform_sdk_contracts import declaration_inventory, normalize


class ContractExtractionTests(unittest.TestCase):
    def test_discards_nested_executable_bodies_and_keeps_abi_fields(self):
        source = '''
libc :: #library,system "libc";
TEXT :: "literal  double spacing";
Record :: struct {
    field: u32 = 7;
    run :: (self: *Record, n: s32 = 3) -> s32 {
        nested :: () { print("DO_NOT_EMIT { // }"); }
        forbidden := 999;
        return forbidden;
    }
}
sdk :: (value: *Record) -> s32 #foreign libc "native_symbol";
'''
        out, report = normalize(source, Path("Sample.jai"))
        self.assertIn("field: u32 = 7;", out)
        self.assertIn('TEXT :: "literal  double spacing";', out)
        self.assertIn('sdk :: (value: *Record) -> s32 #foreign libc "native_symbol";', out)
        self.assertIn('libc :: #system_library "libc";', out)
        self.assertIn("run :: (self: *Record, n: s32 = 3) -> s32 #foreign Native_Adapters", out)
        self.assertNotIn("DO_NOT_EMIT", out)
        self.assertNotIn("forbidden", out)
        self.assertNotIn("nested", out)
        self.assertEqual(report["source_procedure_bodies_discarded"], 1)
        self.assertFalse(report["reference_body_reuse"])

    def test_excludes_shipped_libraries_and_foreign_users(self):
        source = '''
foreign_binary :: #library,no_dll,link_always "x64/original_shipped_binary";
execute_original :: () #foreign foreign_binary;
sdk :: #system_library "kernel32";
GetLastError :: () -> u32 #foreign sdk;
'''
        out, report = normalize(source, Path("Sample.jai"))
        self.assertNotIn("original_shipped_binary", out)
        self.assertNotIn("execute_original", out)
        self.assertIn("GetLastError", out)
        self.assertEqual(len(report["excluded"]), 2)

    def test_excludes_compile_time_implementations_and_raw_templates(self):
        source = '''
template :: #string END
DO_NOT_EMIT_TEMPLATE
END
Record :: struct {
    #insert #run,stallable generate_reference_implementation();
    field: u64;
}
'''
        out, report = normalize(source, Path("Sample.jai"))
        self.assertNotIn("DO_NOT_EMIT_TEMPLATE", out)
        self.assertNotIn("generate_reference_implementation", out)
        self.assertNotIn("#run", out)
        self.assertIn("field: u64;", out)
        self.assertEqual(len(report["excluded"]), 2)

    def test_inventory_ignores_parameter_names(self):
        records = declaration_inventory("T :: struct { value: u32; fn: (argument: u32) -> s32 #c_call; } native :: (parameter: *T) -> s32 #foreign libc;")
        self.assertEqual([record["name"] for record in records], ["T", "value", "fn", "native"])


if __name__ == "__main__":
    unittest.main()
