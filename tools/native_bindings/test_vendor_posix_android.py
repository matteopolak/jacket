import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/native_bindings'))
from vendor_records import record_shape, assertions
from generate import generate_facade
from spec import parse_spec


class AndroidHeaderTests(unittest.TestCase):
    def test_desugared_lp64_types_preserve_field_order_and_layout(self):
        record = {'name':'Example','tagUsed':'struct','completeDefinition':True,'inner':[
            {'kind':'FieldDecl','name':'small','type':{'qualType':'short'}},
            {'kind':'FieldDecl','name':'count','type':{'qualType':'kernel_count','desugaredQualType':'unsigned long'}}]}
        shape = record_shape(record)
        self.assertEqual((shape['size'],shape['alignment']),(16,8))
        self.assertEqual([f['offset'] for f in shape['fields']],[0,8])
        self.assertLess(shape['code'].index('small:'),shape['code'].index('count:'))
        self.assertIn('offsetof(struct Example, count) == 8',assertions([shape]))

    def test_incomplete_anonymous_and_unknown_native_types_are_rejected(self):
        cases = [
            {'name':'Example','tagUsed':'struct'},
            {'name':'Example','tagUsed':'struct','completeDefinition':True,'inner':[
                {'kind':'FieldDecl','type':{'qualType':'int'}}]},
            {'name':'Example','tagUsed':'struct','completeDefinition':True,'inner':[
                {'kind':'FieldDecl','name':'value','type':{'qualType':'UnresolvedNativeType'}}]},
        ]
        for record in cases:
            with self.assertRaises(ValueError):
                record_shape(record)

    def test_header_delegate_only_supplies_declared_target_names(self):
        spec = parse_spec('''# POSIX
## Public declaration inventory
- [OS == .ANDROID && CPU == .X64] `iovec :: struct`
''')
        code, ledger = generate_facade('POSIX',spec)
        self.assertIn('iov_base: *void;',code)
        self.assertNotIn('pollfd ::',code)
        self.assertEqual(ledger[0]['status'],'declared')
        self.assertEqual(ledger[0]['native_shape_delegates'][0]['cpu'],'x64')

    def test_reviewed_shape_takes_precedence_over_native_headers(self):
        spec = parse_spec('''# POSIX
## Public declaration inventory
- [OS == .ANDROID && CPU == .X64] `iovec :: struct`
## Public aggregate shapes and enum values
### `iovec` [OS == .ANDROID && CPU == .X64]
```jai
iovec :: struct { explicit_public_field: s32; }
```
''')
        code, ledger = generate_facade('POSIX',spec)
        self.assertIn('explicit_public_field',code)
        self.assertNotIn('iov_base',code)
        self.assertNotIn('native_shape_delegates',ledger[0])


if __name__ == '__main__':
    unittest.main()
