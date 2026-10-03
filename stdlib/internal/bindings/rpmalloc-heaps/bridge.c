/* Independent canonical facade. The hash-pinned C SDK uses reviewed build
 * fixes. Public records are allocations owned here, never native heap/span
 * aliases. */
#include "runtime-dimensions.h"
/* Establish the Windows SDK's C inline linkage before rpmalloc's generic
 * FORCEINLINE default; otherwise SDK helpers acquire external definitions in
 * every allocator profile when these objects are statically linked together. */
#if defined(_WIN32)
#include <windows.h>
/* rpmalloc's static helpers require its own inline qualifier, not the
 * SDK's extern-inline qualifier. The platform declarations are complete. */
#undef FORCEINLINE
#endif
#include "rpmalloc.c"
/* The SDK defines the selected cache defaults before the public mirror. */
#include "public-layout.h"
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if !PLATFORM_WINDOWS
#include <sys/mman.h>
#endif

_Static_assert(SIZE_CLASS_COUNT == 126 && LARGE_CLASS_COUNT == 63,
               "SDK class dimensions require a reviewed canonical adapter");
_Static_assert(MAX_THREAD_SPAN_CACHE == 400 &&
                   MAX_THREAD_SPAN_LARGE_CACHE == 100,
               "SDK cache dimensions cannot overrun reviewed public records");

#if defined(BINDING_RPMALLOC_FACADE_PREFIX)
#define CONCAT_INNER(a, b) a##b
#define CONCAT(a, b) CONCAT_INNER(a, b)
#define API(name) CONCAT(BINDING_RPMALLOC_FACADE_PREFIX, name)
#elif ENABLE_STATISTICS
#define API(name) binding_rpmalloc_facade_##name
#else
#define API(name) binding_rpmalloc_nostats_facade_##name
#endif

#include "runtime-domain-post.h"
#include "runtime-storage.h"

/* Actual post-SDK-preprocessing build identity. Initialization rejects a
 * library whose options differ from its canonical Jai import instance. */
uint32_t API(build_flags)(void) {
  return (8u << 16) | (!!ENABLE_STATISTICS << 0) |
         (!!BINDING_RPMALLOC_RAW << 1) |
         (!!BINDING_RPMALLOC_PUBLIC_HEAPS << 2) | (!!ENABLE_GLOBAL_CACHE << 3) |
         (!!ENABLE_THREAD_CACHE << 4) | (!!ENABLE_ASSERTS << 5) |
         (!!ENABLE_VALIDATE_ARGS << 6) | (!!ENABLE_ADAPTIVE_THREAD_CACHE << 7);
}

typedef struct HeapView HeapView;
typedef struct SpanView SpanView;
struct SpanView {
  Public_Span value;
  span_t *native;
  HeapView *owner;
  SpanView *next;
  uint64_t epoch;
  uint64_t membership_epoch;
  size_t extent;
  unsigned rank;
};
struct HeapView {
  Public_Heap value;
  heap_t *native;
  HeapView *next;
  uint64_t epoch;
  int acquired;
  unsigned mutating;
  int orphan;
};
#include "runtime-metadata.h"
#include "runtime-live.h"

static int within(const void *pointer, const void *base, size_t size) {
  return pointer && (uintptr_t)pointer >= (uintptr_t)base &&
         (uintptr_t)pointer - (uintptr_t)base < size;
}
static void retire_unmap(void *address, size_t size, size_t offset,
                         size_t release) {
#if !PLATFORM_WINDOWS && !DISABLE_UNMAP
  if (!release && !binding_disable_unmap &&
      native_unmap == _rpmalloc_unmap_os) {
    // Public policy selects an exact advice value, rather than the SDK's
    // platform-preferred FREE_REUSABLE/PAGEOUT fallback chain.
    int advice = retained_advice ? MADV_FREE : MADV_DONTNEED;
    if (madvise(address, size, advice)) {
      rpmalloc_assert(0, "Failed to advise retained allocator pages");
    }
    return;
  }
#endif
  native_unmap(address, size, offset, release);
}
void API(install)(unsigned advice) {
  if (advice > 1)
    abort();
  retained_advice = advice;
  if (_memory_config.memory_unmap != retire_unmap) {
    native_unmap = _memory_config.memory_unmap;
    _memory_config.memory_unmap = retire_unmap;
  }
}

void *API(enter)(BindingDomain *domain) {
  void *previous = API(select_domain)(domain);
  if (!depth++) {
    while (atomic_flag_test_and_set_explicit(&lease, memory_order_acquire)) {
    }
    ++active_calls;
  }
  return previous;
}
unsigned API(pause)(void) {
  unsigned previous = depth;
  if (previous) {
    depth = 0;
    atomic_flag_clear_explicit(&lease, memory_order_release);
  }
  return previous;
}
void API(resume)(unsigned previous) {
  if (previous) {
    while (atomic_flag_test_and_set_explicit(&lease, memory_order_acquire)) {
    }
    depth = previous;
  }
}
static void reclaim(void) {
  SpanView **span_link = &span_views;
  while (*span_link) {
    SpanView *span = *span_link;
    if (!span->native) {
      *span_link = span->next;
      binding_metadata_return_span(span);
    } else
      span_link = &span->next;
  }
  HeapView **heap_link = &views;
  while (*heap_link) {
    HeapView *heap = *heap_link;
    if (!heap->native) {
      *heap_link = heap->next;
      for (SpanView *span = span_views; span; span = span->next)
        if (span->owner == heap) {
          span->owner = NULL;
          span->value.heap = NULL;
        }
      binding_metadata_return_heap(heap);
    } else
      heap_link = &heap->next;
  }
}
static HeapView *find_heap_view(heap_t *native) {
  if (!native)
    return NULL;
  for (HeapView *view = views; view; view = view->next)
    if (view->native == native)
      return view;
  return NULL;
}
static HeapView *bind_heap_view(heap_t *native, HeapView *view) {
  view->native = native;
  view->next = views;
  views = view;
  return view;
}
static HeapView *heap_view(heap_t *native) {
  if (!native)
    return NULL;
  HeapView *view = find_heap_view(native);
  return view ? view : bind_heap_view(native, binding_metadata_heap());
}
static HeapView *owned(Public_Heap *public_heap) {
  for (HeapView *view = views; view; view = view->next)
    if (&view->value == public_heap && view->native &&
        (view->acquired || view->native == get_thread_heap_raw()))
      return view;
  if (!public_heap)
    return NULL;
  if (_memory_config.error_callback)
    _memory_config.error_callback(
        "rpmalloc facade: invalid or released heap handle");
  else
    fputs("rpmalloc facade: invalid or released heap handle\n", stderr);
  abort();
}
static void begin_mutation(HeapView *view) {
  if (view->mutating) {
    if (_memory_config.error_callback)
      _memory_config.error_callback(
          "rpmalloc facade: concurrent or reentrant heap mutation");
    abort();
  }
  view->mutating = 1;
}
void *API(thread_mutation)(void) {
  HeapView *view = heap_view(get_thread_heap_raw());
  if (view)
    begin_mutation(view);
  return view;
}
void API(end_mutation)(void *token) {
  if (token)
    ((HeapView *)token)->mutating = 0;
}
void API(thread_finalized)(void *token) {
  if (token) {
    HeapView *view = token;
    view->mutating = 0;
    view->acquired = 0;
  }
}
void API(require_quiescent)(void) {
  if (active_calls != 1) {
    if (_memory_config.error_callback)
      _memory_config.error_callback(
          "rpmalloc facade: finalize during an active allocator operation");
    abort();
  }
}
int API(valid_request)(size_t size, size_t alignment) {
  /* The SDK stores huge-allocation page counts in uint32_t. Reject requests
   * that its native header cannot represent, before calling mmap or realloc. */
  if (!_memory_page_size || alignment >= _memory_span_size ||
      (alignment && (alignment & (alignment - 1))) ||
      size >
          SIZE_MAX - SPAN_HEADER_SIZE - alignment - (_memory_page_size - 1)) {
    errno = EINVAL;
    return 0;
  }
  size_t pages = (size + SPAN_HEADER_SIZE + alignment + _memory_page_size - 1) /
                 _memory_page_size;
  if (pages > UINT32_MAX) {
    errno = EINVAL;
    return 0;
  }
  return 1;
}
static void deferred_free(span_t *span, void *memory) {
  _rpmalloc_stat_add64(&_deallocation_counter, 1);
  if (span->size_class < SIZE_CLASS_COUNT) {
    _rpmalloc_stat_inc_free(span->heap, span->size_class);
    if (span->flags & SPAN_FLAG_ALIGNED_BLOCKS) {
      uintptr_t offset = (uintptr_t)memory - (uintptr_t)span - SPAN_HEADER_SIZE;
      memory = (void *)((uintptr_t)memory - offset % span->block_size);
    }
    _rpmalloc_deallocate_defer_small_or_medium(span, memory);
  } else
    _rpmalloc_deallocate_defer_free_span(span->heap, span);
}
void API(free_any)(void *memory) {
  if (!memory)
    return;
  span_t *span = (span_t *)((uintptr_t)memory & _memory_span_mask);
  HeapView *view = heap_view(span->heap);
  if (view->mutating)
    deferred_free(span, memory);
  else
    rpmalloc_heap_free(view->native, memory);
}
static Public_Heap *heap_pointer(heap_t *native) {
  for (HeapView *view = views; native && view; view = view->next)
    if (view->native == native && view->epoch == facade_epoch)
      return &view->value;
  return NULL;
}
static SpanView *span_view(HeapView *owner, span_t *native) {
  if (!native)
    return NULL;
  for (SpanView *view = span_views; view; view = view->next)
    if (view->native == native)
      return view;
  SpanView *view = binding_metadata_span();
  view->native = native;
  view->owner = owner;
  view->next = span_views;
  span_views = view;
  return view;
}
/* rank 1: initialized master header; 2: whole-span cache; 3: live allocation.
 * Uninitialized reserve headers are represented from their owning reserve,
 * never read as native span_t objects. Inactive allocation-only fields are
 * zero. */
static void clear_span(Public_Span *out) {
  out->free_list = NULL;
  out->block_count = out->size_class = out->free_list_limit = out->used_count =
      0;
  out->list_size = out->block_size = out->span_count = out->total_spans = 0;
  out->offset_from_master = out->remaining_spans = out->align_offset = 0;
  out->free_list_deferred = NULL;
  out->flags = 0;
  out->heap = NULL;
  out->next = out->prev = NULL;
}
static Public_Span *span_pointer(HeapView *owner, span_t *native,
                                 unsigned rank) {
  SpanView *view = span_view(owner, native);
  if (!view)
    return NULL;
  if (view->epoch == facade_epoch && view->rank >= rank)
    return &view->value;
  Public_Span *out = &view->value;
  clear_span(out);
  view->epoch = facade_epoch;
  view->rank = rank;
  view->owner = owner;
  out->heap = &owner->value;
  out->span_count = native->span_count;
  int huge = rank == 3 && native->size_class == SIZE_CLASS_HUGE;
  out->flags = huge ? 0 : native->flags;
  if (out->flags & SPAN_FLAG_MASTER) {
    out->total_spans = native->total_spans;
    out->remaining_spans = atomic_load32(&native->remaining_spans);
    out->align_offset = native->align_offset;
  } else if (out->flags & SPAN_FLAG_SUBSPAN) {
    out->offset_from_master = native->offset_from_master;
  }
  view->extent = (size_t)out->span_count * _memory_span_size;
  if (rank == 3) {
    out->size_class = native->size_class;
    if (native->size_class < SIZE_CLASS_COUNT) {
      out->free_list = native->free_list;
      out->block_count = native->block_count;
      out->free_list_limit = native->free_list_limit;
      out->used_count = native->used_count;
      out->list_size = native->list_size;
      out->block_size = native->block_size;
      out->free_list_deferred = atomic_load_ptr(&native->free_list_deferred);
    } else if (native->size_class == SIZE_CLASS_HUGE) {
      /* Huge headers initialize only these fields; flags are not initialized.
       */
      out->flags = 0;
      out->total_spans = out->offset_from_master = out->remaining_spans = 0;
      out->align_offset = native->align_offset;
      view->extent = (size_t)native->span_count * _memory_page_size;
    }
  }
  return out;
}
static Public_Span *list(HeapView *owner, span_t *native) {
  Public_Span *head = NULL, *previous = NULL;
  while (native) {
    span_t *next = native->next;
    Public_Span *out = span_pointer(owner, native, 3);
    span_view(owner, native)->membership_epoch = facade_epoch;
    out->prev = previous;
    out->next = NULL;
    if (previous)
      previous->next = out;
    else
      head = out;
    previous = out;
    native = next;
  }
  if (head)
    head->prev = previous;
  return head;
}
static void snapshot(HeapView *view) {
  heap_t *in = view->native;
  Public_Heap *out = &view->value;
  view->epoch = facade_epoch;
  out->owner_thread = in->owner_thread;
  out->full_span_count = (int64_t)in->full_span_count;
  out->spans_reserved = in->spans_reserved;
  out->child_count = atomic_load32(&in->child_count);
  out->id = in->id;
  out->finalize = in->finalize;
  out->next_heap = heap_pointer(in->next_heap);
  out->next_orphan = view->orphan ? heap_pointer(in->next_orphan) : NULL;
  out->master_heap = heap_pointer(in->master_heap);
  for (unsigned i = 0; i < SIZE_CLASS_COUNT; ++i) {
    out->size_class[i].free_list = in->size_class[i].free_list;
    out->size_class[i].partial_span =
        list(view, in->size_class[i].partial_span);
    out->size_class[i].cache = span_pointer(view, in->size_class[i].cache, 2);
#if !BINDING_RPMALLOC_RAW || BINDING_RPMALLOC_PUBLIC_HEAPS
    out->full_span[i] = list(view, in->full_span[i]);
#else
    (void)list(view, in->full_span[i]);
#endif
  }
#if !BINDING_RPMALLOC_RAW || BINDING_RPMALLOC_PUBLIC_HEAPS
  out->large_huge_span = list(view, in->large_huge_span);
#else
  (void)list(view, in->large_huge_span);
#endif
  out->span_reserve_master =
      in->spans_reserved ? span_pointer(view, in->span_reserve_master, 1)
                         : NULL;
  out->span_reserve = NULL;
  if (in->spans_reserved && in->span_reserve) {
    SpanView *reserve = span_view(view, in->span_reserve);
    if (reserve->epoch != facade_epoch) {
      clear_span(&reserve->value);
      reserve->epoch = facade_epoch;
      reserve->rank = 0;
      reserve->owner = view;
      reserve->value.heap = out;
      reserve->value.flags = SPAN_FLAG_SUBSPAN;
      reserve->value.span_count = in->spans_reserved;
      reserve->value.offset_from_master =
          (uint32_t)(((uintptr_t)in->span_reserve -
                      (uintptr_t)in->span_reserve_master) /
                     _memory_span_size);
      reserve->extent = (size_t)in->spans_reserved * _memory_span_size;
    }
    out->span_reserve = &reserve->value;
  }
  out->span_free_deferred = NULL;
  span_t *deferred = atomic_load_ptr(&in->span_free_deferred);
  Public_Span *previous_deferred = NULL;
  while (deferred) {
    Public_Span *span = span_pointer(view, deferred, 3);
    if (previous_deferred)
      previous_deferred->free_list = span;
    else
      out->span_free_deferred = span;
    previous_deferred = span;
    deferred = deferred->free_list;
  }
  if (previous_deferred)
    previous_deferred->free_list = NULL;
#if ENABLE_THREAD_CACHE
  out->span_cache.count = (int64_t)in->span_cache.count;
  for (unsigned i = 0; i < MAX_THREAD_SPAN_CACHE; ++i)
    out->span_cache.span[i] =
        i < in->span_cache.count ? span_pointer(view, in->span_cache.span[i], 2)
                                 : NULL;
  for (unsigned i = 0; i < LARGE_CLASS_COUNT - 1; ++i) {
    out->span_large_cache[i].count = (int64_t)in->span_large_cache[i].count;
    for (unsigned j = 0; j < MAX_THREAD_SPAN_LARGE_CACHE; ++j)
      out->span_large_cache[i].span[j] =
          j < in->span_large_cache[i].count
              ? span_pointer(view, in->span_large_cache[i].span[j], 2)
              : NULL;
  }
#endif
#if ENABLE_ADAPTIVE_THREAD_CACHE || ENABLE_STATISTICS
  for (unsigned i = 0; i < LARGE_CLASS_COUNT; ++i) {
    out->span_use[i].current = atomic_load32(&in->span_use[i].current);
    out->span_use[i].high = in->span_use[i].high;
#if ENABLE_STATISTICS
#define SPAN_COUNTER(field)                                                    \
  out->span_use[i].field = atomic_load32(&in->span_use[i].field)
    SPAN_COUNTER(spans_deferred);
    SPAN_COUNTER(spans_to_global);
    SPAN_COUNTER(spans_from_global);
    SPAN_COUNTER(spans_to_cache);
    SPAN_COUNTER(spans_from_cache);
    SPAN_COUNTER(spans_to_reserved);
    SPAN_COUNTER(spans_from_reserved);
    SPAN_COUNTER(spans_map_calls);
#undef SPAN_COUNTER
#endif
  }
#endif
#if ENABLE_STATISTICS
  for (unsigned i = 0; i < SIZE_CLASS_COUNT + 1; ++i) {
#define SIZE_COUNTER(field)                                                    \
  out->size_class_use[i].field = atomic_load32(&in->size_class_use[i].field)
    SIZE_COUNTER(alloc_current);
    SIZE_COUNTER(alloc_total);
    SIZE_COUNTER(free_total);
    SIZE_COUNTER(spans_current);
    SIZE_COUNTER(spans_to_cache);
    SIZE_COUNTER(spans_from_cache);
    SIZE_COUNTER(spans_from_reserved);
    SIZE_COUNTER(spans_map_calls);
#undef SIZE_COUNTER
    out->size_class_use[i].alloc_peak = in->size_class_use[i].alloc_peak;
    out->size_class_use[i].spans_peak = in->size_class_use[i].spans_peak;
    out->size_class_use[i].unused = in->size_class_use[i].unused;
  }
  out->thread_to_global = atomic_load64(&in->thread_to_global);
  out->global_to_thread = atomic_load64(&in->global_to_thread);
#endif
}
static void publish(void) {
  if (!views)
    return;
  ++facade_epoch;
  for (HeapView *view = views; view; view = view->next)
    view->orphan = 0;
  for (size_t i = 0; i < HEAP_ARRAY_SIZE; ++i)
    for (heap_t *native = _memory_heaps[i]; native; native = native->next_heap)
      heap_view(native)->epoch = facade_epoch;
  for (heap_t *native = _memory_orphan_heaps; native;
       native = native->next_orphan)
    heap_view(native)->orphan = 1;
  for (heap_t *native = _memory_first_class_orphan_heaps; native;
       native = native->next_orphan)
    heap_view(native)->orphan = 1;
  /* All link targets are registered before any record is published. */
  for (HeapView *view = views; view; view = view->next)
    if (view->epoch == facade_epoch)
      snapshot(view);
    else {
      view->acquired = 0;
      view->value.finalize = 1;
    }
}
void API(leave)(void *previous) {
  if (!depth)
    abort();
  if (depth == 1) {
    publish();
    if (--active_calls == 0)
      reclaim();
    depth = 0;
    atomic_flag_clear_explicit(&lease, memory_order_release);
  } else
    --depth;
  if (binding_selected_domain && !active_calls)
    binding_drop_empty_thread_slot(binding_selected_domain);
  API(restore_domain)(previous);
}
void API(reset)(void) {
  span_views = NULL;
  views = NULL;
  binding_metadata_destroy(binding_domain_current());
  facade_epoch = 0;
  native_unmap = NULL;
  retained_advice = 0;
  binding_name_provider = NULL;
  ++binding_domain_current()->generation;
}
Public_Heap *API(acquire)(void) {
  /* The SDK's public acquire assumes success; use its checked internal result.
   */
  if (!_rpmalloc_initialized) {
    errno = EINVAL;
    return NULL;
  }
  /* Every actual SDK heap creation reserves typed publication capacity first.
   * Acquiring an existing orphan therefore requires no new metadata. */
  heap_t *native = _rpmalloc_heap_allocate(1);
  if (!native)
    return NULL;
  native->owner_thread = 0;
  _rpmalloc_stat_inc(&_memory_active_heaps);
  HeapView *view = heap_view(native);
  view->acquired = 1;
  return &view->value;
}
Public_Heap *API(current)(void) {
  HeapView *view = heap_view(get_thread_heap_raw());
  return view ? &view->value : NULL;
}
void API(release)(Public_Heap *heap) {
  HeapView *view = owned(heap);
  if (!view)
    return;
  begin_mutation(view);
  rpmalloc_heap_release(view->native);
  view->mutating = 0;
  view->acquired = 0;
}
void *API(alloc)(Public_Heap *heap, size_t alignment, size_t size) {
  if (!API(valid_request)(size, alignment))
    return NULL;
  HeapView *view = owned(heap);
  if (!view)
    return NULL;
  begin_mutation(view);
  void *result =
      alignment ? rpmalloc_heap_aligned_alloc(view->native, alignment, size)
                : rpmalloc_heap_alloc(view->native, size);
  view->mutating = 0;
  return result;
}
void *API(calloc)(Public_Heap *heap, size_t alignment, size_t num,
                  size_t size) {
  if ((num && size > SIZE_MAX / num) ||
      !API(valid_request)(num * size, alignment)) {
    errno = EINVAL;
    return NULL;
  }
  HeapView *view = owned(heap);
  if (!view)
    return NULL;
  begin_mutation(view);
  void *result = alignment ? rpmalloc_heap_aligned_calloc(view->native,
                                                          alignment, num, size)
                           : rpmalloc_heap_calloc(view->native, num, size);
  view->mutating = 0;
  return result;
}
void *API(realloc)(Public_Heap *heap, void *memory, size_t alignment,
                   size_t size, unsigned flags) {
  if (!API(valid_request)(size, alignment))
    return NULL;
  HeapView *view = owned(heap);
  if (!view)
    return NULL;
  begin_mutation(view);
  void *result = alignment
                     ? rpmalloc_heap_aligned_realloc(view->native, memory,
                                                     alignment, size, flags)
                     : rpmalloc_heap_realloc(view->native, memory, size, flags);
  view->mutating = 0;
  return result;
}
void API(free)(Public_Heap *heap, void *memory) {
  if (heap)
    (void)owned(heap);
  API(free_any)(memory);
}
void API(free_all)(Public_Heap *heap) {
  HeapView *view = owned(heap);
  if (view) {
    begin_mutation(view);
    rpmalloc_heap_free_all(view->native);
    view->mutating = 0;
  }
}
void API(set_current)(Public_Heap *heap) {
  if (!heap) {
    HeapView *previous = heap_view(get_thread_heap_raw());
    if (previous) {
      begin_mutation(previous);
      rpmalloc_heap_thread_set_current(NULL);
      previous->mutating = 0;
      previous->acquired = 0;
    }
    return;
  }
  HeapView *view = owned(heap);
  if (view) {
    begin_mutation(view);
    heap_t *previous = get_thread_heap_raw();
    HeapView *previous_view =
        previous && previous != view->native ? heap_view(previous) : NULL;
    if (previous_view)
      begin_mutation(previous_view);
    rpmalloc_heap_thread_set_current(view->native);
    view->mutating = 0;
    if (previous_view)
      previous_view->mutating = 0;
    for (HeapView *other = views; other; other = other->next)
      if (other->native == previous && other != view)
        other->acquired = 0;
  }
}
int API(contains)(Public_Heap *heap, const void *memory, Public_Span *span) {
  if (!memory || (!heap && !span))
    return 0;
  for (HeapView *view = views; view; view = view->next) {
    if (heap && &view->value != heap)
      continue;
    if (view->epoch != facade_epoch)
      continue;
    for (SpanView *candidate = span_views; candidate;
         candidate = candidate->next) {
      if (candidate->owner != view)
        continue;
      if (span && &candidate->value != span)
        continue;
      if (heap && candidate->membership_epoch != facade_epoch)
        continue;
      if (candidate->epoch == facade_epoch &&
          (uintptr_t)memory >= (uintptr_t)candidate->native &&
          (uintptr_t)memory - (uintptr_t)candidate->native < candidate->extent)
        return 1;
    }
  }
  return 0;
}
void *API(native_heap)(Public_Heap *heap) {
  HeapView *view = owned(heap);
  return view ? view->native : NULL;
}
#include "heap-snapshot.inc"
#include "layout-probes.inc"
