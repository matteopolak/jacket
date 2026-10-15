import sys
from pathlib import Path
import unittest
import hashlib
from contextlib import ExitStack
import tempfile
import json
import re

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools/native_bindings'))
from vendor_rpmalloc import native_type, record
from verify import ROOT, place_private_unit
from generate import condition_for
from spec import Declaration


class NativeAllocatorTranslationTests(unittest.TestCase):
    def test_raw_alias_conditions_do_not_capture_aggregate_declarations(self):
        self.assertEqual(condition_for('rpmalloc',Declaration('Heap :: heap_t','')), 'USE_RAW')
        self.assertEqual(condition_for('rpmalloc',Declaration('heap_acquire :: rpmalloc_heap_acquire','RPMALLOC_FIRST_CLASS_HEAPS')), 'USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS)')
        self.assertEqual(condition_for('rpmalloc',Declaration('rpmalloc_config_t :: struct','')), '')
        self.assertEqual(condition_for('rpmalloc',Declaration('RPMALLOC_CONFIGURABLE :: CONFIGURABLE','')), '')

    def test_raw_and_rewrite_procedure_views_keep_their_mode(self):
        for signature in ('rpmalloc_initialize :: () -> _int',
                          'rpmalloc :: inline (size: size_t) -> *void',
                          'dump_statistics :: (file: *void)'):
            self.assertEqual(condition_for('rpmalloc',Declaration(signature,'')), 'USE_RAW')
        self.assertEqual(condition_for('rpmalloc',Declaration('rpmalloc :: inline (size: s64) -> *void','')), '!USE_RAW')
        self.assertEqual(condition_for('rpmalloc',Declaration('dump_statistics :: (builder: *void) #expand','else branch')), '!USE_RAW && (!ENABLE_STATISTICS)')
        self.assertEqual(condition_for('rpmalloc',Declaration('rpfree :: inline (ptr: *void)','')), '')

    def test_unlabeled_raw_heap_views_still_require_first_class_heaps(self):
        signature = 'heap_alloc :: inline (heap: *rpmalloc_heap_t, size: size_t) -> *void'
        self.assertEqual(condition_for('rpmalloc',Declaration(signature,'')),
                         'USE_RAW && RPMALLOC_FIRST_CLASS_HEAPS')
        self.assertEqual(condition_for('rpmalloc',Declaration(signature,'RPMALLOC_FIRST_CLASS_HEAPS')),
                         'USE_RAW && (RPMALLOC_FIRST_CLASS_HEAPS)')

    def test_native_statistics_profiles_have_separate_symbol_domains(self):
        metadata = json.loads((ROOT/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())
        profiles = metadata['native_profiles']
        self.assertEqual(len(profiles),6)
        self.assertEqual(profiles['binding_rpmalloc_sdk']['options']['ENABLE_STATISTICS'],1)
        self.assertEqual(profiles['binding_rpmalloc_sdk_nostats']['options']['ENABLE_STATISTICS'],0)
        self.assertEqual(len({p['symbol_prefix'] for p in profiles.values()}),6)
        self.assertEqual(len({p['facade_prefix'] for p in profiles.values()}),6)
        code = (ROOT/'stdlib/internal/bindings/rpmalloc-sdk/module.jai').read_text()
        for profile in profiles.values():
            self.assertEqual(profile['options']['BINDING_RPMALLOC_RAW'],int(profile['raw']))
            for entry in metadata['functions']:
                if profile['symbol_prefix']:
                    self.assertIn('"'+profile['symbol_prefix']+entry['name']+'"',code)

    def test_source_only_sdk_export_is_retained_in_both_heap_profiles(self):
        metadata = json.loads((ROOT/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())
        entry = next(f for f in metadata['functions'] if f['name']=='rpmalloc_set_main_thread')
        self.assertEqual(entry['origin'],'official_c_source_definition')
        for exports in metadata['native_storage']['source_export_profiles'].values():
            self.assertIn('rpmalloc_set_main_thread',exports)

    def test_native_program_invokes_every_pinned_sdk_entry_point(self):
        metadata = json.loads((ROOT/'stdlib/internal/bindings/rpmalloc-sdk/provenance.json').read_text())
        source = (ROOT/'tests/native-bindings/rpmalloc-native-sdk.jai').read_text()
        calls = set(re.findall(r'\b(\w+)\s*\(',source))
        self.assertEqual({f['name'] for f in metadata['functions']} - calls,set())

    def test_private_unit_staging_preserves_source_and_dependency_bytes(self):
        with tempfile.TemporaryDirectory(dir=ROOT/'.work/tmp') as directory:
            work = Path(directory)
            source = work/'unit.jai'
            source.write_bytes(b'// independent private unit\nmain :: () {}\n')
            stdlib = work/'stdlib'
            public = stdlib/'rpmalloc/module.jai'
            public.parent.mkdir(parents=True)
            public.write_bytes(b'// facade must remain untouched\n')
            hashes = {}
            with ExitStack() as cleanup:
                unit = place_private_unit('rpmalloc-native-sdk',source,stdlib,hashes,cleanup)
                self.assertEqual(unit.read_bytes(),source.read_bytes())
                dependency = stdlib/'rpmalloc/generated/rpmalloc-sdk-layout.jai'
                original = ROOT/'tests/native-bindings/generated/rpmalloc-sdk-layout.jai'
                self.assertEqual(dependency.read_bytes(),original.read_bytes())
                self.assertEqual(hashes[str(unit.relative_to(stdlib))],hashlib.sha256(source.read_bytes()).hexdigest())
            self.assertFalse(unit.exists())
            self.assertFalse(dependency.exists())
            self.assertEqual(public.read_bytes(),b'// facade must remain untouched\n')

    def test_callbacks_have_c_abi_and_pointer_width_types(self):
        self.assertEqual(native_type('void *(*)(size_t, size_t *)'),
                         '#type (arg0: u64, arg1: *u64) -> *void #c_call')
        self.assertEqual(native_type('void (*)(const char *)'),
                         '#type (arg0: *u8) #c_call')
        self.assertEqual(native_type('const rpmalloc_config_t *'), '*rpmalloc_config_t')

    def test_unknown_target_dependent_scalars_and_variadics_are_rejected(self):
        for text in ('unsigned long', 'long', 'void (*)(int, ...)'):
            with self.assertRaises(ValueError):
                native_type(text)

    def test_named_array_of_anonymous_record_keeps_nested_layout(self):
        inner = dict(kind='RecordDecl', tagUsed='struct', completeDefinition=True,
                     range={'begin': {'offset': 10}},
                     inner=[dict(kind='FieldDecl', name='count', type={'qualType':'size_t'})])
        field = dict(kind='FieldDecl', name='values', type={'qualType':'struct (unnamed at sdk:1:1)[2]'},
                     range={'begin': {'offset': 10}})
        outer = dict(tagUsed='struct', completeDefinition=True, inner=[inner, field])
        code, probes = record(outer, 'Independent')
        self.assertIn('values: [2] struct {', code)
        self.assertEqual(probes, [('Independent', 'values', 'values'),
                                  ('Independent', 'values[0].count', 'values[0].count')])
        field['range']['begin']['offset'] = 11
        with self.assertRaises(ValueError):
            record(outer, 'Independent')

    def test_union_and_bitfields_are_rejected_instead_of_flattened(self):
        with self.assertRaises(ValueError):
            record(dict(tagUsed='union', completeDefinition=True), 'Union')
        with self.assertRaises(ValueError):
            record(dict(tagUsed='struct', completeDefinition=True,
                        inner=[dict(kind='FieldDecl', name='bits', isBitfield=True)]), 'Bits')
