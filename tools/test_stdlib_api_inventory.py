import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from library_source_inventory import tokens
from stdlib_api_inventory import changed_paths, compare, declarations, sha


class ContractInventoryTests(unittest.TestCase):
    def scan(self, source):
        return declarations(tokens(source))

    def test_body_locals_and_private_helpers_are_not_public_contracts(self):
        found = self.scan('f :: (x: int) -> int { local := x; return local; } #scope_file; hidden :: 2;')
        self.assertEqual([d['name'] for d in found], ['f'])
        self.assertEqual(found[0]['implementation_form'], 'source-body')

    def test_bodies_do_not_change_contract_but_defaults_and_fields_do(self):
        one = self.scan('f :: (x := 2) -> int { return x; }')[0]
        two = self.scan('f :: (x := 2) -> int { return x + 1; }')[0]
        changed = self.scan('f :: (x := 3) -> int { return x; }')[0]
        self.assertEqual(one['contract_sha256'], two['contract_sha256'])
        self.assertNotEqual(one['contract_sha256'], changed['contract_sha256'])
        left = self.scan('R :: struct { x: int; y: u8; }')[0]
        right = self.scan('R :: struct { y: u8; x: int; }')[0]
        self.assertNotEqual(left['contract_sha256'], right['contract_sha256'])

    def test_foreign_locator_does_not_require_original_implementation(self):
        original = self.scan('f :: (x: int) -> int #foreign Old;')[0]
        authored = self.scan('f :: (x: int) -> int { return x; }')[0]
        self.assertEqual(original['contract_sha256'], authored['contract_sha256'])
        self.assertEqual(original['implementation_form'], 'foreign-declaration')

    def test_nested_enum_and_operators_are_inventoried(self):
        found = self.scan('R :: struct { K :: enum u8 { A :: 2; B; C; } using #as base: int; } operator + :: (a: R, b: R) -> R { return a; }')
        self.assertEqual([d['name'] for d in found], ['R', 'operator+'])
        self.assertEqual([d['name'] for d in found[0]['members']], ['K', 'base'])
        self.assertEqual([d['name'] for d in found[0]['members'][0]['members']], ['A', 'B', 'C'])

    def test_aggregate_field_defaults_are_contracts(self):
        left = self.scan('R :: struct { default: V = .{1, 2}; other: int; }')[0]
        right = self.scan('R :: struct { default: V = .{1, 3}; other: int; }')[0]
        self.assertEqual([d['name'] for d in left['members']], ['default', 'other'])
        self.assertNotEqual(left['contract_sha256'], right['contract_sha256'])

    def test_named_result_aggregate_default_is_not_an_empty_body(self):
        left = self.scan('f :: () -> result := Error.{} { return .{}; }')[0]
        right = self.scan('f :: () -> result := Error.{7} { return .{}; }')[0]
        self.assertEqual(left['implementation_form'], 'source-body')
        self.assertEqual(right['implementation_form'], 'source-body')
        self.assertNotEqual(left['contract_sha256'], right['contract_sha256'])

    def test_missing_overload_does_not_count_as_equal_api(self):
        reference = {'module': 'M', 'origin': 'historical-reference',
                     'contracts': self.scan('f :: (x: int) {} f :: (x: string) {}')}
        authored = {'origin': 'authored-default', 'contracts': self.scan('f :: (x: int) {}')}
        result = compare(reference, authored)
        self.assertEqual(result['identical_lexical_contracts'], 1)
        self.assertEqual(len(result['missing_or_different_contracts']), 1)
        self.assertEqual(result['behavior_acceptance'], 'not-established')

    def test_snapshot_detects_added_removed_and_edited_sources(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            stable, edited, removed, added = [root / name for name in ('stable', 'edited', 'removed', 'added')]
            for path in (stable, edited, removed):
                path.write_bytes(b'before')
            before = {path: sha(path.read_bytes()) for path in (stable, edited, removed)}
            edited.write_bytes(b'after')
            removed.unlink()
            added.write_bytes(b'new')
            with patch('stdlib_api_inventory.ROOT', root):
                self.assertEqual(changed_paths(before, {stable, edited, added}), ['added', 'edited', 'removed'])


if __name__ == '__main__':
    unittest.main()
