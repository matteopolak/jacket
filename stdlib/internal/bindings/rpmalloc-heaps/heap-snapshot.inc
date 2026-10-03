/* Immutable diagnostic capture. Formatting may allocate on the allocator
 * being described, so it must happen after releasing the native operation
 * lease and must never follow live metadata pointers from the capture. */
#if ENABLE_STATISTICS
typedef struct HeapCounters {
  uint64_t header[6];
  int32_t spans[LARGE_CLASS_COUNT][10];
  int32_t sizes[SIZE_CLASS_COUNT + 1][11];
} HeapCounters;
typedef struct HeapSnapshot {
  size_t count;
  HeapCounters *heaps;
  rpmalloc_global_statistics_t global;
} HeapSnapshot;

void *API(snapshot_create)(void) {
  (void)heap_view(get_thread_heap_raw());
  publish();
  size_t count = 0;
  for (HeapView *view = views; view; view = view->next)
    if (view->native && view->epoch == facade_epoch)
      ++count;
  if (count > SIZE_MAX / sizeof(HeapCounters)) {
    errno = ENOMEM;
    return NULL;
  }
  HeapSnapshot *snapshot = calloc(1, sizeof(*snapshot));
  if (!snapshot)
    return NULL;
  if (count) {
    snapshot->heaps = calloc(count, sizeof(*snapshot->heaps));
    if (!snapshot->heaps) {
      free(snapshot);
      return NULL;
    }
  }
  snapshot->count = count;
  rpmalloc_global_statistics(&snapshot->global);
  size_t index = 0;
  for (HeapView *view = views; view; view = view->next) {
    if (!view->native || view->epoch != facade_epoch)
      continue;
    heap_t *in = view->native;
    HeapCounters *out = &snapshot->heaps[index++];
    out->header[0] = (uint64_t)(int64_t)in->id;
    out->header[1] = in->owner_thread;
    out->header[2] = in->full_span_count;
    out->header[3] = in->spans_reserved;
    out->header[4] = atomic_load64(&in->thread_to_global);
    out->header[5] = atomic_load64(&in->global_to_thread);
    for (unsigned i = 0; i < LARGE_CLASS_COUNT; ++i) {
      out->spans[i][0] = atomic_load32(&in->span_use[i].current);
      out->spans[i][1] = in->span_use[i].high;
      out->spans[i][2] = atomic_load32(&in->span_use[i].spans_deferred);
      out->spans[i][3] = atomic_load32(&in->span_use[i].spans_to_global);
      out->spans[i][4] = atomic_load32(&in->span_use[i].spans_from_global);
      out->spans[i][5] = atomic_load32(&in->span_use[i].spans_to_cache);
      out->spans[i][6] = atomic_load32(&in->span_use[i].spans_from_cache);
      out->spans[i][7] = atomic_load32(&in->span_use[i].spans_to_reserved);
      out->spans[i][8] = atomic_load32(&in->span_use[i].spans_from_reserved);
      out->spans[i][9] = atomic_load32(&in->span_use[i].spans_map_calls);
    }
    for (unsigned i = 0; i <= SIZE_CLASS_COUNT; ++i) {
      out->sizes[i][0] = atomic_load32(&in->size_class_use[i].alloc_current);
      out->sizes[i][1] = in->size_class_use[i].alloc_peak;
      out->sizes[i][2] = atomic_load32(&in->size_class_use[i].alloc_total);
      out->sizes[i][3] = atomic_load32(&in->size_class_use[i].free_total);
      out->sizes[i][4] = atomic_load32(&in->size_class_use[i].spans_current);
      out->sizes[i][5] = in->size_class_use[i].spans_peak;
      out->sizes[i][6] = atomic_load32(&in->size_class_use[i].spans_to_cache);
      out->sizes[i][7] = atomic_load32(&in->size_class_use[i].spans_from_cache);
      out->sizes[i][8] =
          atomic_load32(&in->size_class_use[i].spans_from_reserved);
      out->sizes[i][9] = atomic_load32(&in->size_class_use[i].spans_map_calls);
      out->sizes[i][10] = in->size_class_use[i].unused;
    }
  }
  return snapshot;
}
void API(snapshot_release)(void *pointer) {
  HeapSnapshot *snapshot = pointer;
  if (snapshot) {
    free(snapshot->heaps);
    free(snapshot);
  }
}
uint64_t API(snapshot_count)(const void *pointer) {
  return ((const HeapSnapshot *)pointer)->count;
}
uint64_t API(snapshot_header)(const void *pointer, size_t heap,
                              unsigned field) {
  const HeapSnapshot *snapshot = pointer;
  if (heap >= snapshot->count || field >= 6)
    abort();
  return snapshot->heaps[heap].header[field];
}
const int32_t *API(snapshot_counters)(const void *pointer, size_t heap,
                                      unsigned family) {
  const HeapSnapshot *snapshot = pointer;
  if (heap >= snapshot->count || family > 1)
    abort();
  return family ? &snapshot->heaps[heap].sizes[0][0]
                : &snapshot->heaps[heap].spans[0][0];
}
const rpmalloc_global_statistics_t *API(snapshot_global)(const void *pointer) {
  return &((const HeapSnapshot *)pointer)->global;
}
#endif
