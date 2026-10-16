/* Independent SDK storage/lifecycle probe. Original public C types come from
 * the pinned SDK; no public Jai pointers or guessed layout reach native calls.
 */
#include <assert.h>
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>

static unsigned fail_allocation;
static unsigned rejected_allocations;
static void *storage_calloc(size_t count, size_t width) {
  if (fail_allocation && !--fail_allocation) {
    // Ask the real libc allocator to reject an overflowing allocation. Only
    // the request is redirected; no invented return value/error is supplied.
    volatile size_t overflow_count = SIZE_MAX;
    void *result = calloc(overflow_count, 2);
    assert(!result && errno == ENOMEM);
    ++rejected_allocations;
    return result;
  }
  return calloc(count, width);
}
#define calloc storage_calloc
#include "bridge.c"
#undef calloc

static void exercise(uint64_t buckets, uint64_t batch, uint64_t multiplier) {
  assert(API(configure_dimensions)(buckets, batch, multiplier) == 0);
  assert(rpmalloc_initialize() == 0);
  assert(binding_heap_count == buckets);
  assert(rpmalloc_config()->span_map_count == batch);
  assert(binding_cache_multiplier == multiplier);
  assert(API(configure_dimensions)(1, 1, 1) == EINVAL);
#if ENABLE_GLOBAL_CACHE
  for (size_t i = 1; i < LARGE_CLASS_COUNT; ++i)
    assert(_memory_span_cache[i].span - _memory_span_cache[i - 1].span ==
           (ptrdiff_t)(multiplier * MAX_THREAD_SPAN_CACHE));
#endif
  void *blocks[129];
  for (size_t i = 0; i < 129; ++i) {
    blocks[i] = rpmalloc((i + 1) * 4096);
    assert(blocks[i]);
    memset(blocks[i], 71, (i + 1) * 4096);
  }
  for (size_t i = 0; i < 129; ++i) {
    assert(((unsigned char *)blocks[i])[i * 4096] == 71);
    rpfree(blocks[i]);
  }
  rpmalloc_finalize();
}

int main(void) {
  exercise(17, 3, 2);
  heap_t **previous_heaps = _memory_heaps;
#if ENABLE_GLOBAL_CACHE
  span_t **previous_spans = binding_cache_storage;
#endif
  assert(API(configure_dimensions)(UINT64_MAX, 64, 8) == 0);
  assert(rpmalloc_initialize() == ENOMEM);
  assert(!_rpmalloc_initialized);
  assert(_memory_heaps == previous_heaps && binding_heap_count == 17);
#if ENABLE_GLOBAL_CACHE
  assert(binding_cache_storage == previous_spans);
  assert(API(configure_dimensions)(53, 96, UINT64_MAX) == 0);
  assert(rpmalloc_initialize() == ENOMEM);
  assert(binding_cache_storage == previous_spans);
#endif
  assert(API(configure_dimensions)(53, 96, 3) == 0);
  fail_allocation = 1;
  assert(rpmalloc_initialize() == ENOMEM);
  assert(rejected_allocations == 1);
  assert(_memory_heaps == previous_heaps && binding_heap_count == 17);
#if ENABLE_GLOBAL_CACHE
  assert(binding_cache_storage == previous_spans);
  fail_allocation = 2;
  assert(rpmalloc_initialize() == ENOMEM);
  assert(rejected_allocations == 2);
  assert(_memory_heaps == previous_heaps && binding_heap_count == 17);
  assert(binding_cache_storage == previous_spans);
#endif
  rpmalloc_finalize(); // Failed init rollback is safe and idempotent.
  exercise(53, 96, 3);
  exercise(1, 1, 1);
  return 0;
}
