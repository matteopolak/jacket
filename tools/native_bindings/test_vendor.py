import unittest

from vendor_context import c_enums


class VendorEnumTests(unittest.TestCase):
    def test_implicit_members_and_alias_dependencies_are_preserved(self):
        header = '''typedef enum { UNKNOWN = -1, FIRST, SECOND, ALIAS = SECOND } Example;
typedef enum Tag { FLAG_A = 1U << 0, FLAG_B = FLAG_A << 1 } Flags;'''
        result = c_enums(header)
        self.assertIn('FIRST;', result)
        self.assertIn('ALIAS :: SECOND;', result)
        self.assertIn('FLAG_A :: 1 << 0;', result)
        self.assertIn('Example :: enum s32', result)
        self.assertIn('Flags :: enum u32', result)


if __name__ == '__main__':
    unittest.main()
