import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]/'tools/native_bindings'))
from rpmalloc_context import rewrite_records, render_context

ROOT = Path(__file__).resolve().parents[2]


class ReviewedAllocatorContextTests(unittest.TestCase):
    def setUp(self):
        self.spec = (ROOT/'docs/specs/stdlib/modules/rpmalloc.md').read_text()

    def test_complete_context_field_counts_and_exact_span_order(self):
        records = rewrite_records(self.spec)
        self.assertEqual({name: len(value) for name, value in records.items()}, {
            'Heap': 21, 'Heap_Size_Class': 3, 'Span_Cache': 2,
            'Span_Large_Cache': 2, 'Span': 17, 'Span_Use': 10,
            'Size_Class_Use': 11,
        })
        self.assertEqual([x[0] for x in records['Span']], [
            'free_list', 'block_count', 'size_class', 'free_list_limit',
            'used_count', 'list_size', 'block_size', 'span_count', 'total_spans',
            'offset_from_master', 'remaining_spans', 'align_offset',
            'free_list_deferred', 'flags', 'heap', 'next', 'prev',
        ])
        self.assertEqual(next(x[1] for x in records['Span'] if x[0]=='remaining_spans'), 's32')

    def test_raw_and_rewrite_cache_overlay_order_remain_distinct(self):
        code = render_context(self.spec)
        rewrite, raw = code.split('#if USE_RAW {')
        self.assertLess(rewrite.index('full_span:'), rewrite.index('span_cache:'))
        self.assertLess(raw.index('span_cache:'), raw.index('full_span:'))
        self.assertIn('#if RPMALLOC_FIRST_CLASS_HEAPS', raw)
        self.assertNotIn(':: struct {}', code)
        self.assertIn('#scope_module', code)
        self.assertNotIn('#scope_export', code)

    def test_new_untyped_members_are_rejected_instead_of_dropped(self):
        broken = self.spec.replace('and `cache: *Span`.', 'and `cache: *Span`, `unknown_untyped_member`.', 1)
        with self.assertRaises(ValueError):
            rewrite_records(broken)


if __name__ == '__main__':
    unittest.main()
