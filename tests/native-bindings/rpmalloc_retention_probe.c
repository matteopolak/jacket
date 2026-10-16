/* Observe the real SDK OS release route; never replace a kernel result. */
#include <assert.h>
#include <stdio.h>
#include <sys/mman.h>
static unsigned release_calls;
static int observed_munmap(void *address, size_t length) {
  ++release_calls;
  return munmap(address, length);
}
#define munmap observed_munmap
#include "bridge.c"
#undef munmap

static unsigned exercise(unsigned policy) {
  assert(API(configure_dimensions)(17, 3, 1) == 0);
  assert(API(configure_policy)(policy) == 0);
  release_calls = 0;
  assert(rpmalloc_initialize() == 0);
  API(install)(0);
  void *blocks[320];
  for (size_t i = 0; i < 320; ++i) {
    blocks[i] = rpmalloc(131072);
    assert(blocks[i]);
    memset(blocks[i], 71, 131072);
  }
  for (size_t i = 0; i < 320; ++i) {
    assert(((unsigned char *)blocks[i])[0] == 71);
    rpfree(blocks[i]);
  }
  rpmalloc_thread_finalize(1);
  unsigned before_finalize = release_calls;
  rpmalloc_global_statistics_t statistics;
  rpmalloc_global_statistics(&statistics);
  rpmalloc_finalize();
  fprintf(stderr, "policy=%u early releases=%u total releases=%u cached=%zu\n",
          policy, before_finalize, release_calls, statistics.cached);
  if (policy & 4u)
    assert(release_calls == 0);
  else
    assert(release_calls > before_finalize);
  return before_finalize;
}
int main(void) {
  unsigned limited = exercise(0);
  unsigned unlimited = exercise(2);
  assert(limited > unlimited);
  exercise(6);
  exercise(1); // Linux option has no page-discovery effect on this Mac host.
  assert(API(configure_policy)(4) == EINVAL);
  assert(API(configure_policy)(8) == EINVAL);
  return 0;
}
