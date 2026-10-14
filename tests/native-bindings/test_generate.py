import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "native_bindings"))
from generate import generate_facade, split_arguments, authored_signatures, normalized_signature
from spec import parse_spec


class GeneratorTests(unittest.TestCase):
    def test_foreign_prototype_before_record_is_not_an_authored_body(self):
        text = '''native :: () #foreign lib "call";
Record :: struct { value: u32; }
wrapper :: (value: u32) -> u32 { return value; }
'''
        signatures = authored_signatures(text)
        self.assertEqual(signatures, {normalized_signature('wrapper :: (value: u32) -> u32')})
        self.assertEqual(authored_signatures(text.replace('"call";', '"call"')), signatures)

    def test_native_sdl_wm_shape_uses_explicit_safe_adapter(self):
        spec = parse_spec('''# SDL
## Public declaration inventory
- `SDL_GetWindowWMInfo :: (window: *SDL_Window, info: *SDL_SysWMinfo) -> SDL_bool #foreign SDL2`
''')
        code, ledger = generate_facade('SDL', spec)
        self.assertNotIn('SDL_GetWindowWMInfo ::', code)
        self.assertEqual(ledger[0]['status'], 'implemented_manual')
        self.assertEqual(ledger[0]['native_abi_adapter'], 'SDL_GetWindowWMInfo')
        changed = parse_spec('''# SDL
## Public declaration inventory
- `SDL_GetWindowWMInfo :: (window: *SDL_Window, different_name: *SDL_SysWMinfo) -> SDL_bool #foreign SDL2`
''')
        code, ledger = generate_facade('SDL', changed)
        self.assertNotIn('SDL_GetWindowWMInfo ::', code)
        self.assertEqual(ledger[0]['status'], 'unresolved')

    def test_arguments_handle_nested_function_types(self):
        self.assertEqual(split_arguments('a: s32, cb: #type (x: s32, y: s32) #c_call, b: [2]u32'),
                         ['a: s32', 'cb: #type (x: s32, y: s32) #c_call', 'b: [2]u32'])

    def test_forwarding_preserves_parameter_names_defaults_and_return(self):
        spec = parse_spec('''# d3d11
## Public declaration inventory
- `IFace_Run :: inline (this: *IFace, flags: u32 = 7) -> s32`
- `vtable :: (obj: *IFace) -> *IFace_VTable`
''')
        code, ledger = generate_facade('d3d11', spec)
        self.assertIn('flags: u32 = 7', code)
        self.assertIn('return this.vtable.Run(this, flags);', code)
        self.assertIn('return obj.vtable;', code)
        self.assertEqual([d['status'] for d in ledger], ['forwarded', 'forwarded'])

    def test_implicit_enum_member_is_not_dropped(self):
        spec = parse_spec('''# Example
## Public declaration inventory
- `Tag :: enum u32`
## Public type members
#### `Tag` (enum)
- `ZERO :: 0`
- `ONE`
- `LAST :: ONE`
''')
        code, ledger = generate_facade('Example', spec)
        self.assertIn('Tag :: enum u32 {', code)
        self.assertIn('ONE;', code)
        self.assertEqual(ledger[0]['status'], 'declared')

    def test_unknown_wrapper_remains_explicitly_unresolved(self):
        spec = parse_spec('''# Example
## Public declaration inventory
- `mystery :: (x: s32) -> bool`
''')
        code, ledger = generate_facade('Example', spec)
        self.assertNotIn('mystery ::', code)
        self.assertEqual(ledger[0]['status'], 'unresolved')

    def test_string_span_and_value_reference_forwarding(self):
        spec = parse_spec('''# ImGui
## Public declaration inventory
- `Draw :: (pos: *Vec2, text_begin: *u8, text_end: *u8) #foreign native`
- `Draw :: (pos: Vec2, text: string) #no_context`
''')
        code, ledger = generate_facade('ImGui', spec)
        self.assertIn('Draw(*pos, text.data, text.data + text.count)', code)
        self.assertEqual(ledger[1]['status'], 'implemented')

    def test_independent_top_level_order_preserves_semantic_sequences(self):
        spec = parse_spec('''# Example
## Public declaration inventory
- `zebra :: (value: s32) #foreign native`
- `storage_z: s32 = 5`
- `BETA :: 2`
- `zebra :: (value: u64) #foreign native`
- `storage_a: s32 = storage_z`
- `ALPHA :: 1`
- `alpha :: () #foreign native`
- `Tag :: enum u32`
## Public type members
#### `Tag` (enum)
- `ZED :: 10`
- `AYE`
''')
        code, _ = generate_facade('Example', spec)
        self.assertLess(code.index('Tag ::'), code.index('ALPHA ::'))
        self.assertLess(code.index('ALPHA ::'), code.index('BETA ::'))
        self.assertLess(code.index('storage_z:'), code.index('storage_a:'))
        self.assertLess(code.index('alpha ::'), code.index('zebra ::'))
        self.assertLess(code.index('value: s32'), code.index('value: u64'))
        self.assertLess(code.index('ZED ::'), code.index('AYE;'))

    def test_cpp_methods_on_same_line_are_forwarded_and_accounted(self):
        spec = parse_spec('''# nvtt
## Public declaration inventory
- `Compressor :: struct`
## Public type members
### `Compressor`
```jai
Compressor :: struct {
run :: (this: *Compressor, options: *Options) -> bool #foreign libnvtt; run :: (this: *Compressor, options: Options) -> bool #no_context;
}
```
''')
        code, ledger = generate_facade('nvtt', spec)
        self.assertIn('return run(this, *options);', code)
        self.assertEqual([m['status'] for m in ledger[0]['members']], ['declared', 'implemented'])

    def test_imgui_format_overloads_select_native_pointer_signature(self):
        spec = parse_spec('''# ImGui
## Public declaration inventory
- `TreeNode_CFormat :: (str_id: *u8, fmt: *u8, __args: ..Any) -> bool #foreign imgui "string_symbol"`
- `TreeNode_CFormat :: (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #foreign imgui "pointer_symbol"`
- `TreeNode :: (ptr_id: *void, fmt: string, __args: ..Any) -> bool`
''')
        code, ledger = generate_facade('ImGui',spec)
        self.assertIn('Native_Format :: #type (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #c_call',code)
        self.assertIn('Varargs.invoke_bool',code)
        self.assertEqual(ledger[-1]['status'],'implemented')

    def test_authored_coverage_matches_complete_overload_contract(self):
        signatures = authored_signatures('''
write :: (value: string, suffix := "a b") -> bool { return true; }
write :: (value: s32) -> bool { return true; }
''')
        self.assertIn(normalized_signature('write :: (value: string, suffix := "a b") -> bool'),signatures)
        self.assertNotIn(normalized_signature('write :: (text: string, suffix := "a b") -> bool'),signatures)
        self.assertNotIn(normalized_signature('write :: (value: string, suffix := "ab") -> bool'),signatures)
        self.assertNotIn(normalized_signature('write :: (value: u64) -> bool'),signatures)

    def test_com_helper_alias_uses_canonical_receiver_vtable(self):
        spec = parse_spec('''# dxc_compiler
## Public declaration inventory
- `Wide :: Utf16`
- `Wide_GetStringLength :: inline (this: *Utf16) -> u64`
''')
        code, ledger = generate_facade('dxc_compiler',spec)
        self.assertIn('return this.vtable.GetStringLength(this);',code)
        self.assertEqual(ledger[-1]['status'],'forwarded')

    def test_broader_shape_is_valid_and_missing_cpu_shape_stays_unresolved(self):
        page = '''# Example
## Public declaration inventory
- [OS == .MACOS && CPU == .X64] `Record :: struct`
## Public aggregate shapes and enum values
### `Record`
Variant 1 (macOS):
```jai
Record :: struct { value: u64; }
```
'''
        code, ledger = generate_facade('Example',parse_spec(page))
        self.assertIn('value: u64',code)
        self.assertEqual(ledger[0]['status'],'declared')
        code, ledger = generate_facade('Example',parse_spec(page.replace('Variant 1 (macOS):','Variant 1 (macOS, ARM64):')))
        self.assertNotIn('Record ::',code)
        self.assertEqual(ledger[0]['status'],'unresolved')

    def test_partial_shape_keeps_supported_branch_and_records_gap(self):
        spec = parse_spec('''# Example
## Public declaration inventory
- [OS == .MACOS] `Record :: struct`
## Public aggregate shapes and enum values
### `Record`
Variant 1 (macOS, ARM64):
```jai
Record :: struct { value: u64; }
```
''')
        code, ledger = generate_facade('Example',spec)
        self.assertIn('CPU == .ARM64',code)
        self.assertEqual(ledger[0]['status'],'partially_declared')
        self.assertIn('unresolved_shape_condition',ledger[0])


if __name__ == '__main__':
    unittest.main()
