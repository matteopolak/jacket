import sys
from pathlib import Path
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools/native_bindings'))
from vendor_rpmalloc import storage_type


class NativeStorageTranslationTests(unittest.TestCase):
    def test_native_arrays_and_atomic_storage_keep_width_and_nesting(self):
        self.assertEqual(storage_type('span_t *[400]'), '[400] *Span_Storage')
        self.assertEqual(storage_type('heap_size_class_t[126]'), '[126] Size_Class_Storage')
        self.assertEqual(storage_type('atomicptr_t'), '*void')
        self.assertEqual(storage_type('atomic32_t'), 's32')
        self.assertEqual(storage_type('atomic64_t'), 's64')
        self.assertEqual(storage_type('uintptr_t'), 'u64')

    def test_unknown_native_storage_types_are_rejected(self):
        with self.assertRaises(ValueError):
            storage_type('new_unreviewed_native_t')


if __name__ == '__main__':
    unittest.main()
