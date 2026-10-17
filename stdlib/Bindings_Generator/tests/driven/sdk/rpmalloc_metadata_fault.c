/* Darwin test observer. Reject real libc requests; never fabricate SDK results.
 * Native programs only: interpreter allocation belongs to its compiler image.
 */
#define _DARWIN_C_SOURCE
#include <dlfcn.h>
#include <mach-o/dyld.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static _Thread_local unsigned until_failure;
static _Thread_local unsigned denied;
void metadata_fail_abort(uint64_t line) {
  fprintf(stderr, "metadata regression failed at line %llu\n",
          (unsigned long long)line);
  abort();
}
static void *observed_calloc(size_t count, size_t size) {
  Dl_info caller;
  int selected =
      until_failure && dladdr(__builtin_return_address(0), &caller) &&
      (caller.dli_fbase == (void *)_dyld_get_image_header(0) ||
       (caller.dli_fname && strstr(caller.dli_fname, "binding_rpmalloc_sdk")));
  if (selected && !--until_failure) {
    ++denied;
    volatile size_t overflow = SIZE_MAX;
    return calloc(overflow, 2);
  }
  return calloc(count, size);
}
__attribute__((used, section("__DATA,__interpose"))) static const struct {
  const void *replacement;
  const void *original;
} interpose = {(const void *)observed_calloc, (const void *)calloc};
void metadata_fail_begin(uint32_t ordinal) {
  until_failure = ordinal;
  denied = 0;
}
uint32_t metadata_fail_end(void) {
  until_failure = 0;
  return denied;
}
typedef struct Worker {
  void (*run)(void);
} Worker;
static void *run(void *argument) {
  ((Worker *)argument)->run();
  return NULL;
}
int32_t metadata_worker(void (*procedure)(void)) {
  Worker worker = {procedure};
  pthread_t thread;
  int result = pthread_create(&thread, NULL, run, &worker);
  if (result)
    return result;
  return pthread_join(thread, NULL);
}
