/* Typed publication capacity is reserved before SDK ownership mutation. */
#include <assert.h>
struct BindingMetadataChunk {
  BindingMetadataChunk *next;
  HeapView *heaps;
  SpanView *spans;
  size_t heap_count;
  size_t span_count;
};
static void binding_metadata_free_chunk(BindingMetadataChunk *chunk) {
  if (chunk) {
    free(chunk->heaps);
    free(chunk->spans);
    free(chunk);
  }
}
static void binding_metadata_return_heap(HeapView *view) {
  BindingDomain *domain = binding_domain_current();
  memset(view, 0, sizeof(*view));
  view->next = domain->owned_free_heaps;
  domain->owned_free_heaps = view;
  ++domain->owned_free_heap_count;
}
static void binding_metadata_return_span(SpanView *view) {
  BindingDomain *domain = binding_domain_current();
  memset(view, 0, sizeof(*view));
  view->next = domain->owned_free_spans;
  domain->owned_free_spans = view;
  ++domain->owned_free_span_count;
}
static HeapView *binding_metadata_heap(void) {
  BindingDomain *domain = binding_domain_current();
  HeapView *view = domain->owned_free_heaps;
  assert(view && "SDK heap creation must reserve its publication capacity");
  if (!view)
    abort();
  domain->owned_free_heaps = view->next;
  --domain->owned_free_heap_count;
  memset(view, 0, sizeof(*view));
  return view;
}
static SpanView *binding_metadata_span(void) {
  BindingDomain *domain = binding_domain_current();
  SpanView *view = domain->owned_free_spans;
  assert(view && "SDK mapping must reserve its publication capacity");
  if (!view)
    abort();
  domain->owned_free_spans = view->next;
  --domain->owned_free_span_count;
  memset(view, 0, sizeof(*view));
  return view;
}
static int binding_metadata_reserve(BindingMetadataReservation *reservation,
                                    size_t heaps, size_t spans) {
  BindingDomain *domain = binding_domain_current();
  size_t new_heaps = heaps > domain->owned_free_heap_count
                         ? heaps - domain->owned_free_heap_count
                         : 0;
  size_t new_spans = spans > domain->owned_free_span_count
                         ? spans - domain->owned_free_span_count
                         : 0;
  if (new_heaps > SIZE_MAX / sizeof(HeapView) ||
      new_spans > SIZE_MAX / sizeof(SpanView)) {
    errno = ENOMEM;
    return 0;
  }
  BindingMetadataChunk *chunk = NULL;
  if (new_heaps || new_spans) {
    chunk = calloc(1, sizeof(*chunk));
    if (!chunk)
      goto failure;
    chunk->heap_count = new_heaps;
    chunk->span_count = new_spans;
    if (new_heaps) {
      chunk->heaps = calloc(new_heaps, sizeof(*chunk->heaps));
      if (!chunk->heaps)
        goto failure;
    }
    if (new_spans) {
      chunk->spans = calloc(new_spans, sizeof(*chunk->spans));
      if (!chunk->spans)
        goto failure;
    }
  }
  reservation->chunk = chunk;
  for (size_t i = 0; i < heaps - new_heaps; ++i) {
    HeapView *view = binding_metadata_heap();
    view->next = reservation->heaps;
    reservation->heaps = view;
  }
  for (size_t i = 0; i < spans - new_spans; ++i) {
    SpanView *view = binding_metadata_span();
    view->next = reservation->spans;
    reservation->spans = view;
  }
  return 1;
failure:
  binding_metadata_free_chunk(chunk);
  errno = ENOMEM;
  return 0;
}
static void binding_metadata_cancel(BindingMetadataReservation *reservation) {
  while (reservation->heaps) {
    HeapView *view = reservation->heaps;
    reservation->heaps = view->next;
    binding_metadata_return_heap(view);
  }
  while (reservation->spans) {
    SpanView *view = reservation->spans;
    reservation->spans = view->next;
    binding_metadata_return_span(view);
  }
  binding_metadata_free_chunk(reservation->chunk);
  reservation->chunk = NULL;
}
static void binding_metadata_commit(BindingMetadataReservation *reservation) {
  BindingMetadataChunk *chunk = reservation->chunk;
  if (chunk) {
    for (size_t i = 0; i < chunk->heap_count; ++i)
      binding_metadata_return_heap(&chunk->heaps[i]);
    for (size_t i = 0; i < chunk->span_count; ++i)
      binding_metadata_return_span(&chunk->spans[i]);
    chunk->next = binding_domain_current()->owned_metadata_chunks;
    binding_domain_current()->owned_metadata_chunks = chunk;
    reservation->chunk = NULL;
  }
  binding_metadata_cancel(reservation);
}
static void *binding_metadata_map(size_t size, size_t *offset, size_t spans) {
  BindingMetadataReservation reservation = {0};
  if (!binding_metadata_reserve(&reservation, 0, spans))
    return NULL;
  void *memory = _rpmalloc_mmap(size, offset);
  if (memory) {
    binding_metadata_commit(&reservation);
    // Claim every potential native span header for this live SDK mapping.
    // Reserved/cache headers need not be initialized yet: no fields are read.
    // Their capacity remains owned until actual native unmap retirement.
    for (size_t i = 0; i < spans; ++i) {
      SpanView *view = binding_metadata_span();
      view->native = (span_t *)((char *)memory + i * _memory_span_size);
      view->next = span_views;
      span_views = view;
    }
  } else
    binding_metadata_cancel(&reservation);
  return memory;
}
static void
binding_metadata_commit_heaps(BindingMetadataReservation *reservation,
                              void *base, size_t stride, size_t count) {
  binding_metadata_commit(reservation);
  for (size_t i = 0; i < count; ++i) {
    HeapView *view = binding_metadata_heap();
    view->native = (heap_t *)((char *)base + i * stride);
    view->next = views;
    views = view;
  }
}
static void binding_metadata_unmap(void *address, size_t size, size_t release) {
  size_t extent = release ? release : size;
  for (SpanView *view = span_views; view; view = view->next)
    if (view->native && (uintptr_t)view->native >= (uintptr_t)address &&
        (uintptr_t)view->native - (uintptr_t)address < extent) {
      view->epoch = 0;
      if (release)
        view->native = NULL;
    }
  if (release)
    for (HeapView *view = views; view; view = view->next)
      if (view->native && (uintptr_t)view->native >= (uintptr_t)address &&
          (uintptr_t)view->native - (uintptr_t)address < extent) {
        view->native = NULL;
        view->epoch = 0;
        view->acquired = 0;
      }
}
static void binding_metadata_destroy(BindingDomain *domain) {
  BindingMetadataChunk *chunk = domain->owned_metadata_chunks;
  while (chunk) {
    BindingMetadataChunk *next = chunk->next;
    binding_metadata_free_chunk(chunk);
    chunk = next;
  }
  domain->owned_metadata_chunks = NULL;
  domain->owned_free_heaps = NULL;
  domain->owned_free_spans = NULL;
  domain->owned_free_heap_count = domain->owned_free_span_count = 0;
}
