/* Independently authored private runtime policy, before the pinned SDK body.
 * Public module parameters do not alter SDK/public heap or span records. */
#include <stddef.h>
#include <stdint.h>
#include <stdatomic.h>
#include "runtime-callback-state.h"
typedef struct HeapView HeapView;
typedef struct SpanView SpanView;
typedef struct BindingDomain BindingDomain;
typedef struct BindingMetadataChunk BindingMetadataChunk;
typedef struct BindingMetadataReservation {
  HeapView *heaps;
  SpanView *spans;
  BindingMetadataChunk *chunk;
} BindingMetadataReservation;
static int binding_metadata_reserve(BindingMetadataReservation *reservation,
                                    size_t heaps, size_t spans);
static void binding_metadata_commit(BindingMetadataReservation *reservation);
static void
binding_metadata_commit_heaps(BindingMetadataReservation *reservation,
                              void *base, size_t stride, size_t count);
static void binding_metadata_cancel(BindingMetadataReservation *reservation);
static void binding_metadata_destroy(BindingDomain *domain);
static void *binding_metadata_map(size_t size, size_t *offset, size_t spans);
static void binding_metadata_unmap(void *address, size_t size, size_t release);
static int binding_domain_prepare_heap(void);

static char *binding_rpmalloc_live_name(int huge);

#undef HEAP_ARRAY_SIZE
#define HEAP_ARRAY_SIZE binding_heap_count
#undef DEFAULT_SPAN_MAP_COUNT
#define DEFAULT_SPAN_MAP_COUNT binding_default_map_count
#undef GLOBAL_CACHE_MULTIPLIER
#define GLOBAL_CACHE_MULTIPLIER binding_cache_multiplier
#define BINDING_RPMALLOC_RUNTIME_CONFIGURATION 1
static int binding_rpmalloc_prepare_storage(void);
