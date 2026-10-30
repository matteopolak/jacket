"""The numeric modules declare exactly the names their specs list."""

import unittest

from numerics_surface import surface_failures


class NumericsSurface(unittest.TestCase):
    def test_public_names_match_the_specs(self):
        self.assertEqual(surface_failures(), [])


if __name__ == "__main__":
    unittest.main()
