"""Shared source/runtime inventory; feature selection never drops default gates."""
DEFAULT = (
    'rpmalloc-native-sdk', 'rpmalloc-native-storage', 'rpmalloc-heap-marshalling',
    'rpmalloc-native-error-callback',
    'rpmalloc-public-allocation', 'rpmalloc-public-callbacks', 'rpmalloc-public-configurable',
    'rpmalloc-public-raw', 'rpmalloc-public-raw-callbacks', 'rpmalloc-public-profiles',
    'rpmalloc-public-heaps', 'rpmalloc-public-heap-lifecycle',
    'rpmalloc-public-allocator-proc', 'rpmalloc-public-builder-dump', 'rpmalloc-public-dump-failure',
)
RAW_HEAPS = ('rpmalloc-public-raw-heaps', 'rpmalloc-public-raw-heap-lifecycle',
             'rpmalloc-raw-heap-marshalling')
RAW_BASE = ('rpmalloc-raw-base-marshalling',)


def fixtures(first_class_heaps=False):
    return DEFAULT+(RAW_HEAPS if first_class_heaps else RAW_BASE)
