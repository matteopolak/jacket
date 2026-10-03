/* Native SDK code is shared; every public module owns a typed state object. */
typedef struct BindingDomain BindingDomain;
typedef struct BindingThreadSlot BindingThreadSlot;
struct BindingDomain {
#include "runtime-domain-native-fields.inc"
  size_t owned_heap_count;
  size_t owned_default_map_count;
  size_t owned_cache_multiplier;
  uint64_t owned_pending_heap_count;
  uint64_t owned_pending_map_count;
  uint64_t owned_pending_cache_multiplier;
  unsigned owned_pending_policy;
  unsigned owned_unlimited_cache;
  unsigned owned_disable_unmap;
  unsigned owned_linux_huge_pages;
  void (*owned_name_provider)(int32_t, const void **, uint64_t *);
#if ENABLE_GLOBAL_CACHE
  span_t **owned_cache_storage;
#endif
  HeapView *owned_views;
  SpanView *owned_span_views;
  BindingMetadataChunk *owned_metadata_chunks;
  HeapView *owned_free_heaps;
  SpanView *owned_free_spans;
  size_t owned_free_heap_count;
  size_t owned_free_span_count;
  uint64_t owned_epoch;
  atomic_flag owned_lease;
  unsigned owned_active_calls;
  void (*owned_native_unmap)(void *, size_t, size_t, size_t);
  unsigned owned_retained_advice;
  BindingCallbackState owned_callbacks;
  _Atomic uint32_t references;
  uint64_t generation;
  unsigned closed;
  unsigned owned_finalizing;
  unsigned owned_fls_ready;
};
#define BINDING_DOMAIN_INITIALIZER                                             \
  {.owned_heap_count = 47,                                                     \
   .owned_default_map_count = 64,                                              \
   .owned_cache_multiplier = 8,                                                \
   .owned_pending_heap_count = 47,                                             \
   .owned_pending_map_count = 64,                                              \
   .owned_pending_cache_multiplier = 8,                                        \
   .owned_lease = ATOMIC_FLAG_INIT,                                            \
   .owned_callbacks = {.map_lock = ATOMIC_FLAG_INIT},                          \
   .references = 1}
static BindingDomain binding_default_domain = BINDING_DOMAIN_INITIALIZER;
static _Thread_local BindingDomain *binding_selected_domain;
static BindingDomain *binding_domain_current(void) {
  return binding_selected_domain ? binding_selected_domain
                                 : &binding_default_domain;
}
static heap_t **binding_domain_heap_slot(void);
static unsigned *binding_domain_depth_slot(void);
#if defined(_WIN32) && (!defined(BUILD_DYNAMIC_LINK) || !BUILD_DYNAMIC_LINK)
static void NTAPI binding_domain_windows_destructor(void *value);
static void binding_domain_fls_heap(heap_t *heap);
static DWORD binding_domain_fls_allocate(void);
static void binding_domain_fls_free(void);
#endif
#include "runtime-domain-native-aliases.inc"
#define binding_heap_count (binding_domain_current()->owned_heap_count)
#define binding_default_map_count                                              \
  (binding_domain_current()->owned_default_map_count)
#define binding_cache_multiplier                                               \
  (binding_domain_current()->owned_cache_multiplier)
#define binding_pending_heap_count                                             \
  (binding_domain_current()->owned_pending_heap_count)
#define binding_pending_map_count                                              \
  (binding_domain_current()->owned_pending_map_count)
#define binding_pending_cache_multiplier                                       \
  (binding_domain_current()->owned_pending_cache_multiplier)
#define binding_pending_policy (binding_domain_current()->owned_pending_policy)
#define binding_unlimited_cache                                                \
  (binding_domain_current()->owned_unlimited_cache)
#define binding_disable_unmap (binding_domain_current()->owned_disable_unmap)
#define binding_linux_huge_pages                                               \
  (binding_domain_current()->owned_linux_huge_pages)
#define binding_name_provider (binding_domain_current()->owned_name_provider)
#define binding_cache_storage (binding_domain_current()->owned_cache_storage)
#define views (binding_domain_current()->owned_views)
#define span_views (binding_domain_current()->owned_span_views)
#define facade_epoch (binding_domain_current()->owned_epoch)
#define lease (binding_domain_current()->owned_lease)
#define depth (*binding_domain_depth_slot())
#define active_calls (binding_domain_current()->owned_active_calls)
#define native_unmap (binding_domain_current()->owned_native_unmap)
#define retained_advice (binding_domain_current()->owned_retained_advice)
