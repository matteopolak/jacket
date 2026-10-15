"""Independent profile selection requirements for the reviewed cache option."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]/'tools/native_bindings'))
from rpmalloc_profiles import PROFILES


class CacheProfiles(unittest.TestCase):
    def test_thread_and_global_caches_have_independent_native_selections(self):
        for raw, heaps in ((False, True), (True, False), (True, True)):
            for stats in (True, False):
                matching = [p for p in PROFILES if (p['raw'], p['public_heaps'], p['statistics']) ==
                            (raw, heaps, stats)]
                self.assertEqual({(p['options'].get('ENABLE_THREAD_CACHE', 1),
                                   p['options']['ENABLE_GLOBAL_CACHE']) for p in matching},
                                 {(0, 0), (0, 1), (1, 0), (1, 1)})

    def test_each_storage_profile_has_real_global_cache_selections(self):
        for raw, heaps in ((False, True), (True, False), (True, True)):
            for stats in (True, False):
                matching = [p for p in PROFILES if (p['raw'], p['public_heaps'], p['statistics']) ==
                            (raw, heaps, stats)]
                self.assertEqual({p['options'].get('ENABLE_GLOBAL_CACHE', 1) for p in matching}, {0, 1})
                self.assertEqual(len({p['sdk'] for p in matching}), len(matching))
                self.assertEqual(len({p['callback_prefix'] for p in matching}), len(matching))


if __name__ == '__main__':
    unittest.main()
