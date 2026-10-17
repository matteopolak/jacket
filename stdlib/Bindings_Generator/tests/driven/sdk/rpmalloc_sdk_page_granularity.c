/* Independent SDK page-granularity regression. The reviewed public profile
 * permits a span smaller than a page; heap-pool mappings must still be pages.
 */
#include "rpmalloc.c"
#include <assert.h>

int main(void) {
  rpmalloc_config_t config = {0};
  config.page_size = 16384;
  config.span_size = 4096;
  config.span_map_count = 4;
  assert(rpmalloc_initialize_config(&config) == 0);
  rpmalloc_heap_t *heap = rpmalloc_heap_acquire();
  assert(heap);
  void *memory = rpmalloc_heap_alloc(heap, 6000);
  assert(memory);
  rpmalloc_heap_free(heap, memory);
  rpmalloc_heap_release(heap);
  rpmalloc_finalize();
  return 0;
}
