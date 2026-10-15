import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/native_bindings'))
from rpmalloc_heap_facade import public_c_header, layout_probes, mirror_records


class CanonicalHeapTranslationTests(unittest.TestCase):
    def setUp(self):
        self.spec = (ROOT/'docs/specs/stdlib/modules/rpmalloc.md').read_text()

    def test_mirror_keeps_public_span_order_and_width(self):
        code = public_c_header(self.spec)
        span = code.split('struct Public_Span {',1)[1].split('};',1)[0]
        self.assertLess(span.index('remaining_spans'),span.index('free_list_deferred'))
        self.assertLess(span.index('free_list_deferred'),span.index('flags'))
        self.assertIn('sizeof(Public_Span)==96',code)
        self.assertNotIn('typedef heap_t',code)
        self.assertNotIn('typedef span_t',code)

    def test_all_public_fields_have_offset_and_width_probes(self):
        c,jai = layout_probes(self.spec)
        self.assertEqual(c.count('offsetof('),66)
        self.assertEqual(c.count('sizeof(((Public_'),66)
        self.assertIn('canonical_layout_statistics',jai)
        self.assertIn('canonical_layout_no_statistics',jai)
        disabled = jai.split('canonical_layout_no_statistics',1)[1]
        self.assertNotIn('heap.span_use',disabled)
        self.assertNotIn('heap.size_class_use',disabled)
        self.assertIn('heap.span_cache',disabled)

    def test_raw_mirror_retains_its_distinct_order_and_atomic_storage(self):
        raw = mirror_records(self.spec,raw=True)
        rewrite = mirror_records(self.spec)
        raw_heap = [f[0] for f in raw['Heap']]
        rewrite_heap = [f[0] for f in rewrite['Heap']]
        self.assertLess(raw_heap.index('span_cache'),raw_heap.index('full_span'))
        self.assertGreater(rewrite_heap.index('span_cache'),rewrite_heap.index('full_span'))
        self.assertEqual(next(f[1] for f in raw['Span'] if f[0]=='remaining_spans'),'atomic32_t')
        code = public_c_header(self.spec).split('#else\n/* Generated reviewed public layouts',1)[0]
        self.assertIn('Public_Atomic32 remaining_spans;',code)
        self.assertIn('Public_AtomicPtr free_list_deferred;',code)
        probes,jai = layout_probes(self.spec,raw=True)
        self.assertEqual(probes.count('offsetof('),66)
        self.assertEqual(probes.count('sizeof(((Public_'),66)
        self.assertIn('canonical_layout_raw_statistics',jai)


if __name__ == '__main__':
    unittest.main()
