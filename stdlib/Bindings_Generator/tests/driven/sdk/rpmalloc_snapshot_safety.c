/* Independent SDK/snapshot safety and real libc allocation fault injection.
 * The production adapter is compiled unchanged; no public stub or test branch.
 */
#include <assert.h>
#include <stdlib.h>

static long allocation_budget = -1;
static size_t live_allocations;
static void *checked_calloc(size_t count, size_t size) {
  if (allocation_budget == 0)
    return NULL;
  if (allocation_budget > 0)
    --allocation_budget;
  void *result = calloc(count, size);
  if (result)
    ++live_allocations;
  return result;
}
static void checked_free(void *pointer) {
  if (pointer) {
    assert(live_allocations);
    --live_allocations;
  }
  free(pointer);
}
#define calloc checked_calloc
#define free checked_free
#include "../../../../internal/bindings/rpmalloc-heaps/bridge.c"
#undef calloc
#undef free

static void check_acquire_metadata_failure(void) {
  API(enter)();
  size_t before = live_allocations;
  heap_t *first_class = _memory_first_class_orphan_heaps;
  heap_t *ordinary = _memory_orphan_heaps;
  heap_t *buckets[HEAP_ARRAY_SIZE];
  memcpy(buckets, _memory_heaps, sizeof(buckets));
  HeapView *associations = views;
#if ENABLE_STATISTICS
  int32_t active = atomic_load32(&_memory_active_heaps);
#endif
  allocation_budget = 0;
  errno = 0;
  assert(API(acquire)() == NULL);
  assert(errno == ENOMEM);
  allocation_budget = -1;
  assert(live_allocations == before && views == associations);
  assert(first_class == _memory_first_class_orphan_heaps);
  assert(ordinary == _memory_orphan_heaps);
  assert(memcmp(buckets, _memory_heaps, sizeof(buckets)) == 0);
#if ENABLE_STATISTICS
  assert(active == atomic_load32(&_memory_active_heaps));
#endif
  API(leave)();
}

static size_t denied_native_maps;
static void *deny_native_mapping(size_t size, size_t *offset) {
  assert(size && offset);
  ++denied_native_maps;
  return NULL;
}

static void check_acquire_native_failure(void) {
  size_t available = 0;
  for (heap_t *heap = _memory_first_class_orphan_heaps; heap;
       heap = heap->next_orphan)
    ++available;
  Public_Heap **held = checked_calloc(available + 1, sizeof(*held));
  assert(held);
  for (size_t i = 0; i < available; ++i) {
    API(enter)();
    held[i] = API(acquire)();
    API(leave)();
    assert(held[i]);
  }
  size_t baseline = live_allocations;
  API(enter)();
  void *(*original_map)(size_t, size_t *) = _memory_config.memory_map;
  _memory_config.memory_map = deny_native_mapping;
  assert(API(acquire)() == NULL && denied_native_maps > 0);
  _memory_config.memory_map = original_map;
  API(leave)();
  /* The preflight reservation must be freed when the real SDK cannot map a
   * new heap pool; no injected allocator body or fabricated SDK result. */
  assert(live_allocations == baseline);
  for (size_t i = 0; i < available; ++i) {
    API(enter)();
    API(release)(held[i]);
    API(leave)();
  }
  checked_free(held);
}

int main(void) {
  API(enter)();
  assert(rpmalloc_initialize() == 0);
  API(install)();
  API(leave)();
  check_acquire_metadata_failure();
  check_acquire_native_failure();
  API(enter)();
  Public_Heap *heap = API(acquire)();
  assert(heap);
  void *memory = API(alloc)(heap, 0, 512);
  assert(memory);
  ((unsigned char *)memory)[0] = 71;
  API(leave)();
  check_acquire_metadata_failure();
#if ENABLE_STATISTICS
  size_t baseline = live_allocations;
  for (long failure = 0; failure < 2; ++failure) {
    API(enter)();
    allocation_budget = failure;
    assert(API(snapshot_create)() == NULL);
    allocation_budget = -1;
    API(leave)();
    assert(live_allocations == baseline);
    assert(((unsigned char *)memory)[0] == 71);
  }
  API(enter)();
  void *snapshot = API(snapshot_create)();
  assert(snapshot && API(snapshot_count)(snapshot));
  size_t index = 0;
  while (index < API(snapshot_count)(snapshot) &&
         API(snapshot_header)(snapshot, index, 0) !=
             (uint64_t)(int64_t)heap->id)
    ++index;
  assert(index < API(snapshot_count)(snapshot));
  const int32_t *counters = API(snapshot_counters)(snapshot, index, 1);
  int64_t current = 0;
  heap_t *native = owned(heap)->native;
  for (unsigned i = 0; i <= SIZE_CLASS_COUNT; ++i) {
    assert(counters[i * 11] ==
           atomic_load32(&native->size_class_use[i].alloc_current));
    current += counters[i * 11];
  }
  assert(current == 1);
  API(free)(heap, memory);
  API(leave)();
  int64_t retained = 0;
  for (unsigned i = 0; i <= SIZE_CLASS_COUNT; ++i)
    retained += counters[i * 11];
  assert(retained == current);
  API(snapshot_release)(snapshot);
  API(enter)();
  snapshot = API(snapshot_create)();
  assert(snapshot);
  index = 0;
  while (index < API(snapshot_count)(snapshot) &&
         API(snapshot_header)(snapshot, index, 0) !=
             (uint64_t)(int64_t)heap->id)
    ++index;
  assert(index < API(snapshot_count)(snapshot));
  counters = API(snapshot_counters)(snapshot, index, 1);
  current = 0;
  for (unsigned i = 0; i <= SIZE_CLASS_COUNT; ++i)
    current += counters[i * 11];
  assert(current == 0);
  API(snapshot_release)(snapshot);
#else
  API(enter)();
  API(free)(heap, memory);
#endif
  API(release)(heap);
  rpmalloc_finalize();
  API(reset)();
  API(leave)();
  assert(live_allocations == 0);
  return 0;
}
