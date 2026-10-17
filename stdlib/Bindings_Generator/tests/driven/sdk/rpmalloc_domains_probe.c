/* Real pinned SDK/typed adapter domains, no public-pointer/native casts. */
#include <stdlib.h>
static unsigned calloc_failure;
static void *checked_calloc(size_t count, size_t size) {
  if (calloc_failure && !--calloc_failure)
    return NULL;
  return calloc(count, size);
}
#define calloc checked_calloc
#include "bridge.c"
#undef calloc
#include <assert.h>
#include <pthread.h>

static BindingDomain *first_domain;
static BindingDomain *second_domain;
static void *first_memory;
static void *second_memory;

static void *worker(void *unused) {
  (void)unused;
  BindingOperationScope scope;
  API(scope_enter)(first_domain, &scope);
  rpmalloc_thread_initialize();
  assert(rpmalloc_config()->span_map_count == 3);
  assert(((unsigned char *)first_memory)[0] == 71);
  rpfree(first_memory);
  rpmalloc_thread_finalize(1);
  API(scope_leave)(&scope);
  API(scope_enter)(second_domain, &scope);
  rpmalloc_thread_initialize();
  assert(rpmalloc_config()->span_map_count == 96);
  assert(((unsigned char *)second_memory)[0] == 74);
  rpfree(second_memory);
  rpmalloc_thread_finalize(1);
  API(scope_leave)(&scope);
  return NULL;
}
int main(void) {
  calloc_failure = 1;
  assert(!API(domain_create)());
  calloc_failure = 2;
  assert(!API(domain_create)());
  assert(!binding_thread_slots);
  BindingDomain *retry = API(domain_create)();
  assert(retry);
  void *retry_prior = API(enter)(retry);
  API(leave)(retry_prior);
  assert(!binding_thread_slots);
  calloc_failure = 1;
  assert(API(domain_prepare_thread)(retry) == ENOMEM);
  assert(!binding_thread_slots && !retry->sdk_rpmalloc_initialized);
  assert(API(domain_prepare_thread)(retry) == 0);
  API(domain_destroy)(retry);
  assert(!binding_thread_slots);
  first_domain = API(domain_create)();
  second_domain = API(domain_create)();
  assert(first_domain && second_domain && first_domain != second_domain);
  void *prior = API(enter)(first_domain);
  assert(API(configure_dimensions)(17, 3, 2) == 0);
  assert(rpmalloc_initialize() == 0);
  assert(_memory_heap_reserve_count == 3);
  first_memory = rpmalloc(4096);
  assert(first_memory);
  memset(first_memory, 71, 4096);
  heap_t *first_heap = get_thread_heap_raw();
  API(leave)(prior);
  prior = API(enter)(second_domain);
  assert(API(configure_dimensions)(53, 96, 3) == 0);
  assert(rpmalloc_initialize() == 0);
  assert(_memory_heap_reserve_count == 96);
  assert(get_thread_heap_raw() != first_heap);
  second_memory = rpmalloc(4096);
  assert(second_memory && second_memory != first_memory);
  memset(second_memory, 74, 4096);
  API(leave)(prior);
  pthread_t thread;
  assert(pthread_create(&thread, NULL, worker, NULL) == 0);
  assert(pthread_join(thread, NULL) == 0);
  prior = API(enter)(first_domain);
  assert(rpmalloc_config()->span_map_count == 3);
  rpmalloc_finalize();
  API(reset)();
  API(leave)(prior);
  API(domain_destroy)(first_domain);
  prior = API(enter)(second_domain);
  assert(_rpmalloc_initialized && rpmalloc_config()->span_map_count == 96);
  void *memory = rpmalloc(71);
  assert(memory);
  rpfree(memory);
  rpmalloc_finalize();
  API(reset)();
  API(leave)(prior);
  API(domain_destroy)(second_domain);
  // A per-heap reserve cap must not cap the independent global reserve.
  BindingDomain *reserve_domain = API(domain_create)();
  assert(reserve_domain);
  prior = API(enter)(reserve_domain);
  assert(API(configure_dimensions)(17, 3, 2) == 0);
  rpmalloc_config_t config = {0};
  config.span_map_count = 96;
  assert(rpmalloc_initialize_config(&config) == 0);
  assert(_memory_span_map_count == 96 && _memory_heap_reserve_count == 3);
  void *reserved_memory = rpmalloc(71);
  assert(reserved_memory);
  assert(get_thread_heap_raw()->spans_reserved <= 3);
  assert(_memory_global_reserve_count > 3);
  rpfree(reserved_memory);
  void *nested_prior = API(enter)(reserve_domain);
  rpmalloc_finalize();
  API(reset)();
  API(leave)(nested_prior);
  // Canonical owner retirement under Allocator_Proc's remaining outer lease
  // must preserve its slot until leave, then reclaim both slot and domain.
  API(domain_destroy)(reserve_domain);
  assert(reserve_domain->closed && reserve_domain->owned_active_calls == 1);
  API(leave)(prior);
  // Repeated lifecycles must reclaim typed domains and empty TLS associations.
  assert(!binding_thread_slots);
  return 0;
}
