"""Regression fixtures are authored from the public specification formats."""
import importlib.util
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "native_bindings"))
from spec import parse_spec


class SpecTests(unittest.TestCase):
    def test_conditional_shapes_stay_separate(self):
        page = '''# Example
## Public declaration inventory
### Unix-specific declarations
- `Choice :: enum u32`
### Windows-specific declarations
- `Choice :: enum s32`
## Public type members
### Unix-specific type declarations
#### `Choice` (enum)
- `FIRST :: 0`
- `SECOND`
- `ALIAS :: SECOND`
### Windows-specific type declarations
#### `Choice` (enum)
- `FIRST :: 0`
- `SECOND`
- `ALIAS :: SECOND`
'''
        spec = parse_spec(page)
        self.assertEqual([d.condition for d in spec.declarations],
                         ["OS != .WINDOWS", "OS == .WINDOWS"])
        self.assertEqual(len(spec.shapes), 2)
        self.assertIn("SECOND;", spec.shapes[0].code)

    def test_full_shapes_preserve_anonymous_union_and_using_prefix(self):
        code = '''Interface_VTable :: struct {
    using base: Base_VTable;
    union {
        first: u64;
        struct { lo: u32; hi: u32; }
    }
    call: #type (this: *Interface, flags: u32 = 3) -> s32 #c_call;
}'''
        page = '''# Example
## Public declaration inventory
- [OS == .WINDOWS] `Interface_VTable :: struct`
## Public aggregate shapes and enum values
### `Interface_VTable`
```jai
''' + code + '''
```
'''
        spec = parse_spec(page)
        self.assertEqual(spec.shapes[0].code, code)

    def test_list_header_before_full_block_does_not_emit_empty_record(self):
        spec = parse_spec('''# Example
## Public type members
#### `Record` (struct)
```jai
Record :: struct { value: s32; }
```
''')
        self.assertEqual(len(spec.shapes), 1)
        self.assertIn("value: s32", spec.shapes[0].code)

    def test_inherited_prefix_is_first_vtable_member(self):
        spec = parse_spec('''# Example
## Public type members
#### `Child_VTable` (struct)
- Inherited vtable prefix: `using base: Parent_VTable`; slots in order: A, B.
- `C: #type (this: *Child) #cpp_method`
''')
        code = spec.shapes[0].code
        self.assertLess(code.index("using base"), code.index("C:"))

    def test_parameter_fence_is_separate(self):
        spec = parse_spec('''# Example
## Module parameters
```jai
#module_parameters (ENABLED := false)();
```
''')
        self.assertEqual(spec.parameter_blocks, ["#module_parameters (ENABLED := false)();"])

    def test_multiple_types_in_one_fence(self):
        spec = parse_spec('''# Example
## Public type members
#### Flag types
```jai
using Tag :: enum u32 { A; B; }
Record :: struct {
    nested :: enum { X; Y; }
    value: Tag;
}
using Bits :: enum_flags u8 { C :: 1; }
```
''')
        self.assertEqual([s.name for s in spec.shapes], ['Tag', 'Record', 'Bits'])
        self.assertIn('nested :: enum', spec.shapes[1].code)

    def test_gl_metadata_does_not_detach_target_fields(self):
        spec = parse_spec('''# GL
## Public type members
#### `Procs` (struct; `!ALL`)
- `flag: bool`
## Library and ABI metadata
- [OS == .MACOS] `lib :: #library,system "OpenGL"`
### Target-conditional fields
#### [OS == .MACOS]
- `call: () #c_call`
''')
        self.assertEqual(len(spec.shapes), 1)
        self.assertIn('call:', spec.shapes[0].code)
        self.assertNotIn('#library', spec.shapes[0].code)

    def test_parenthetical_platform_availability(self):
        spec = parse_spec('''# GL
## Public declaration inventory
- `create :: () -> *void` (Windows only)
''')
        self.assertEqual(spec.declarations[0].condition, 'OS == .WINDOWS')

    def test_condition_between_shape_name_and_kind(self):
        spec = parse_spec('''# Android
## Public type members
#### `App` [GLUE] (struct)
```jai
App :: struct { state: s64; }
```
#### `App` [!GLUE] (struct)
```jai
App :: struct { state: s32; }
```
#### `Other` (struct)
- `value: u32`
''')
        self.assertEqual([s.condition for s in spec.shapes], ['GLUE', '!GLUE', ''])

    def test_variant_conditions_and_overloads(self):
        page = '''# Example
## Public declaration inventory
- [OS == .LINUX] `State :: struct`
- [OS == .MACOS && CPU == .ARM64] `State :: struct`
- `call :: (state: *State) -> s32 #foreign lib`
- `call :: (state: State) -> s32`
## Public aggregate shapes and enum values
### `State`
Variant 1 (Linux):
```jai
State :: struct { value: s32; }
```
Variant 2 (macOS, ARM64):
```jai
State :: struct { value: s64; }
```
'''
        spec = parse_spec(page)
        self.assertEqual(len(spec.declarations), 4)
        self.assertEqual([s.condition for s in spec.shapes],
                         ["OS == .LINUX", "OS == .MACOS && CPU == .ARM64"])

    def test_contextual_sdk_blocks_are_not_public_shapes(self):
        page = '''# Example
## Public declaration inventory
- `api: *Context = null`
## SDK table shape reachable through exported pointers
```jai
Context :: struct { call: #type () #c_call; }
```
'''
        spec = parse_spec(page)
        self.assertEqual(len(spec.shapes), 0)
        self.assertEqual(len(spec.contextual_blocks), 1)

    def test_condition_labels_do_not_leak_into_module_parameters(self):
        page = '''# Example
## Public declaration inventory
- [FEATURE] `call :: () -> s32`
## Module parameters
- `FEATURE := false`
'''
        spec = parse_spec(page)
        self.assertEqual(len(spec.declarations), 1)
        self.assertEqual(spec.parameters, ["FEATURE := false"])


if __name__ == "__main__":
    unittest.main()
