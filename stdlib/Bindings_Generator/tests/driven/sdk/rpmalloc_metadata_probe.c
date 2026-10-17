/* Independent metadata failure and ownership test against the actual SDK. */
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
static unsigned fail_metadata;
static unsigned metadata_failures;
static void *metadata_calloc(size_t count, size_t size) {
  if (fail_metadata && !--fail_metadata) {
    ++metadata_failures;
    return calloc(SIZE_MAX, 2); /* Actual libc overflow/ENOMEM rejection. */
  }
  return calloc(count, size);
}
#define calloc metadata_calloc
#include "../rpmalloc-callbacks/bridge.c"
#include "bridge.c"
#undef calloc
#include <assert.h>
#include <pthread.h>
static BindingDomain *worker_domain;
static void *handoff;
static void *worker(void *unused) {
  (void)unused;
  BindingOperationScope scope;
  API(scope_enter)(worker_domain, &scope);
  fail_metadata = 1;
  rpmalloc_thread_initialize();
  assert(!rpmalloc_is_thread_initialized() && errno == ENOMEM);
  assert(metadata_failures == 5);
  assert(((unsigned char *)handoff)[0] == 91);
  // Even without a persistent TLS association, free is a real ownership
  // operation. It must not allocate metadata, leak the object or abort.
  API(free_any)(handoff);
  API(scope_leave)(&scope);
  API(scope_enter)(worker_domain, &scope);
  rpmalloc_thread_initialize();
  assert(rpmalloc_is_thread_initialized());
  rpmalloc_thread_finalize(1);
  API(scope_leave)(&scope);
  assert(!binding_thread_slots);
  return NULL;
}

int main(void) {
  for (unsigned failure = 3; failure <= 4; ++failure) {
    BindingDomain *initial = API(domain_create)();
    assert(initial);
    void *before = API(enter)(initial);
    rpmalloc_config_t config = {0};
    config.memory_map = _rpmalloc_mmap_os;
    config.memory_unmap = _rpmalloc_unmap_os;
    fail_metadata = failure; // Tables succeed, then heap capacity fails.
    unsigned failures_before = metadata_failures;
    assert(binding_rpmalloc_initialize(&config, NULL, 0, NULL, 0) == ENOMEM);
    assert(metadata_failures == failures_before + 1);
    assert(!_rpmalloc_initialized && !rpmalloc_is_thread_initialized());
    assert(!views && !span_views && !API(callback_storage)()->maps);
    assert(binding_rpmalloc_initialize(&config, NULL, 0, NULL, 0) == 0);
    binding_rpmalloc_finalize();
    API(reset)();
    API(leave)(before);
    API(domain_destroy)(initial);
  }
  metadata_failures = 0;
  BindingDomain *domain = API(domain_create)();
  assert(domain);
  void *previous = API(enter)(domain);
  assert(rpmalloc_initialize() == 0);
  /* Default-thread/publication capacity must precede native heap mutation.
   * Looking up an already-created native heap cannot require fresh metadata. */
  fail_metadata = 1;
  Public_Heap *heap = API(current)();
  assert(heap && metadata_failures == 0 && fail_metadata == 1);
  API(leave)(previous);
  assert(metadata_failures == 0 && fail_metadata == 1);
  previous = API(enter)(domain);
  int32_t mapped = atomic_load32(&_mapped_pages);
  void *failed = API(alloc)(heap, 0, 1048576);
  assert(!failed && errno == ENOMEM && metadata_failures == 1);
  assert(atomic_load32(&_mapped_pages) == mapped);
  fail_metadata = 2;
  failed = API(alloc)(heap, 0, 1048576);
  assert(!failed && errno == ENOMEM && metadata_failures == 2);
  assert(atomic_load32(&_mapped_pages) == mapped);
  API(leave)(previous);
  assert(metadata_failures == 2);
  previous = API(enter)(domain);
  void *memory = API(alloc)(heap, 0, 33554432);
  assert(memory);
  memset(memory, 71, 33554432);
  API(leave)(previous);
  previous = API(enter)(domain);
  // Exhaust reusable graph capacity through real live native mappings. A
  // reservation failure must be observed only when capacity actually grows.
  void *held[1024];
  size_t held_count = 0;
  fail_metadata = 1;
  while (held_count < 1024) {
    void *value = API(alloc)(heap, 0, 8388608);
    if (!value)
      break;
    held[held_count++] = value;
    ((unsigned char *)value)[0] = 74;
    API(leave)(previous);
    previous = API(enter)(domain);
  }
  assert(held_count < 1024 && metadata_failures == 3 && errno == ENOMEM);
  /* Failure before realloc ownership mutation preserves the original object. */
  fail_metadata = 1;
  failed = API(realloc)(heap, memory, 0, 67108864, 0);
  assert(!failed && errno == ENOMEM && metadata_failures == 4);
  assert(((unsigned char *)memory)[0] == 71 &&
         ((unsigned char *)memory)[33554431] == 71);
  API(leave)(previous);
  previous = API(enter)(domain);
  API(free)(heap, memory);
  for (size_t i = 0; i < held_count; ++i) {
    assert(((unsigned char *)held[i])[0] == 74);
    API(free)(heap, held[i]);
  }
  handoff = API(alloc)(heap, 0, 8388608);
  assert(handoff);
  ((unsigned char *)handoff)[0] = 91;
  API(leave)(previous);
  worker_domain = domain;
  pthread_t thread;
  assert(pthread_create(&thread, NULL, worker, NULL) == 0);
  assert(pthread_join(thread, NULL) == 0);
  previous = API(enter)(domain);
  rpmalloc_finalize();
  API(reset)();
  API(leave)(previous);
  API(domain_destroy)(domain);
  assert(!binding_thread_slots);
  return 0;
}
