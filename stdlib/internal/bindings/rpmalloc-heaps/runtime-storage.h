/* Included after the actual SDK types. Every allocation is private metadata;
 * publish a replacement only after all allocations/representability checks.
 * Finalize has already drained native caches before a reconfiguration. */
#include <errno.h>

static int binding_rpmalloc_prepare_storage(void) {
  if (binding_pending_heap_count > SIZE_MAX / sizeof(heap_t *) ||
      binding_pending_map_count > UINT32_MAX ||
      binding_pending_map_count > SIZE_MAX / (256u * 1024u))
    return ENOMEM;
#if ENABLE_GLOBAL_CACHE
  // Native cache.count is uint32_t, and capacity includes all size classes.
  if (binding_pending_cache_multiplier > UINT32_MAX / MAX_THREAD_SPAN_CACHE ||
      binding_pending_cache_multiplier >
          SIZE_MAX /
              (LARGE_CLASS_COUNT * MAX_THREAD_SPAN_CACHE * sizeof(span_t *)))
    return ENOMEM;
#endif
  heap_t **heaps = calloc((size_t)binding_pending_heap_count, sizeof(*heaps));
  if (!heaps)
    return ENOMEM;
#if ENABLE_GLOBAL_CACHE
  size_t capacity =
      (size_t)binding_pending_cache_multiplier * MAX_THREAD_SPAN_CACHE;
  span_t **spans = calloc(LARGE_CLASS_COUNT * capacity, sizeof(*spans));
  if (!spans) {
    free(heaps);
    return ENOMEM;
  }
#endif
  free(_memory_heaps);
  _memory_heaps = heaps;
#if ENABLE_GLOBAL_CACHE
  free(binding_cache_storage);
  binding_cache_storage = spans;
  for (size_t i = 0; i < LARGE_CLASS_COUNT; ++i)
    _memory_span_cache[i].span = spans + i * capacity;
#endif
  binding_heap_count = (size_t)binding_pending_heap_count;
  binding_default_map_count = (size_t)binding_pending_map_count;
  binding_cache_multiplier = (size_t)binding_pending_cache_multiplier;
  binding_linux_huge_pages = !!(binding_pending_policy & 1u);
  binding_unlimited_cache = !!(binding_pending_policy & 2u);
  binding_disable_unmap = !!(binding_pending_policy & 4u);
  return 0;
}

int32_t API(configure_dimensions)(uint64_t heaps, uint64_t map_count,
                                  uint64_t cache_multiplier) {
  if (!heaps || !map_count || !cache_multiplier || _rpmalloc_initialized)
    return EINVAL;
  binding_pending_heap_count = heaps;
  binding_pending_map_count = map_count;
  binding_pending_cache_multiplier = cache_multiplier;
  return 0;
}

int32_t API(configure_policy)(uint32_t policy) {
  if (_rpmalloc_initialized || policy > 7u ||
      ((policy & 2u) && !ENABLE_GLOBAL_CACHE) ||
      ((policy & 4u) && !(policy & 2u)))
    return EINVAL;
  binding_pending_policy = policy;
  return 0;
}
