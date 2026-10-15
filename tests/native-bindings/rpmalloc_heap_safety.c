/* Independent SDK adapter safety regression, separate from public Jai tests. */
#include "../../stdlib/internal/bindings/rpmalloc-heaps/bridge.c"
#include <assert.h>

static unsigned denied_maps;
static void *deny_mapping(size_t size, size_t *offset) {
  assert(size && offset);
  ++denied_maps;
  return NULL;
}

int main(void) {
  API(enter)();
  assert(rpmalloc_initialize() == 0);
  API(install)();
  API(leave)();
  for (unsigned iteration = 0; iteration < 128; ++iteration) {
    API(enter)();
    Public_Heap *heap = API(acquire)();
    API(leave)();
    assert(heap);
    void *blocks[8];
    for (unsigned i = 0; i < 8; ++i) {
      size_t size = i == 7 ? 8 * 1024 * 1024 : (i + 1) * 513;
      API(enter)();
      blocks[i] = API(alloc)(heap, 0, size);
      API(leave)();
      assert(blocks[i]);
      ((unsigned char *)blocks[i])[0] = (unsigned char)i;
      API(enter)();
      assert(API(contains)(heap, blocks[i], NULL));
      assert(!API(contains)(heap, (void *)(uintptr_t)123, NULL));
      API(leave)();
    }
    for (unsigned i = 0; i < 8; ++i) {
      assert(((unsigned char *)blocks[i])[0] == (unsigned char)i);
      API(enter)();
      API(free)(heap, blocks[i]);
      API(leave)();
    }
    API(enter)();
    API(free_all)(heap);
    API(release)(heap);
    API(leave)();
  }
  /* SDK-private failure oracle: exhaust actual reusable heaps, then deny a new
   * native mapping. No public test assumes a particular SDK pool size. */
  size_t available = 0;
  for (heap_t *heap = _memory_first_class_orphan_heaps; heap;
       heap = heap->next_orphan)
    ++available;
  Public_Heap **held = calloc(available, sizeof(*held));
  assert(held || !available);
  for (size_t i = 0; i < available; ++i) {
    API(enter)();
    held[i] = API(acquire)();
    API(leave)();
    assert(held[i]);
  }
  API(enter)();
  void *(*previous_map)(size_t, size_t *) = _memory_config.memory_map;
  _memory_config.memory_map = deny_mapping;
  assert(API(acquire)() == NULL && denied_maps > 0);
  _memory_config.memory_map = previous_map;
  API(leave)();
  for (size_t i = 0; i < available; ++i) {
    API(enter)();
    API(release)(held[i]);
    API(leave)();
  }
  free(held);
  /* Independent SDK state after enough released heaps to overflow thread
   * batches: cache-disabled builds must not retain spans in a global cache. */
  rpmalloc_global_statistics_t statistics;
  rpmalloc_global_statistics(&statistics);
#if !ENABLE_GLOBAL_CACHE
  assert(statistics.cached == 0);
#endif
  API(enter)();
  rpmalloc_finalize();
  API(reset)();
  API(leave)();
  assert(!views && !span_views && active_calls == 0 && depth == 0);
  return 0;
}
