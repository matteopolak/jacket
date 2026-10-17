/* Independent Darwin syscall observer. Every result comes from real madvise;
 * fault mode asks the kernel to reject an invalid advice value. */
#define _DARWIN_C_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <limits.h>
#include <mach-o/dyld.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

static _Thread_local int watching;
static _Thread_local int force_failure;
static _Atomic uint64_t counts[6];

static int observed_advice(void *address, size_t size, int advice) {
  Dl_info caller;
  int selected =
      watching && dladdr(__builtin_return_address(0), &caller) &&
      (caller.dli_fbase == (void *)_dyld_get_image_header(0) ||
       (caller.dli_fname && strstr(caller.dli_fname, "binding_rpmalloc_sdk")));
  if (selected) {
    unsigned index = advice == MADV_DONTNEED        ? 0
                     : advice == MADV_FREE          ? 1
                     : advice == MADV_FREE_REUSABLE ? 2
                                                    : 5;
    atomic_fetch_add(&counts[index], 1);
  }
  /* dyld leaves the interposing image's own original-symbol calls intact. */
  int result =
      madvise(address, size, selected && force_failure ? INT_MAX : advice);
  if (selected && result) {
    atomic_fetch_add(&counts[3], 1);
    atomic_store(&counts[4], (uint64_t)errno);
  }
  return result;
}

__attribute__((used, section("__DATA,__interpose"))) static const struct {
  const void *replacement;
  const void *original;
} interpose = {(const void *)observed_advice, (const void *)madvise};

void binding_advice_begin(int fail) {
  for (unsigned i = 0; i < 6; ++i)
    atomic_store(&counts[i], 0);
  force_failure = fail;
  watching = 1;
}

void binding_advice_end(void) { watching = 0; }
uint64_t binding_advice_count(unsigned index) {
  if (index >= 6)
    abort();
  return atomic_load(&counts[index]);
}
void binding_advice_report(void) {
  fprintf(
      stderr,
      "advice DONTNEED=%llu FREE=%llu REUSABLE=%llu failures=%llu errno=%llu\n",
      (unsigned long long)atomic_load(&counts[0]),
      (unsigned long long)atomic_load(&counts[1]),
      (unsigned long long)atomic_load(&counts[2]),
      (unsigned long long)atomic_load(&counts[3]),
      (unsigned long long)atomic_load(&counts[4]));
}
