/* Independently authored native test fixture; no public facade declarations. */
#define _POSIX_C_SOURCE 200112L
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <malloc.h>
#include <windows.h>
#else
#include <pthread.h>
#endif

static _Atomic uint64_t counters[5];
static _Atomic uint32_t handoff_next;
void binding_rpmalloc_handoff_reset(void) { atomic_store(&handoff_next, 0); }
uint32_t binding_rpmalloc_handoff_claim(void) {
  return atomic_fetch_add(&handoff_next, 1);
}
void *binding_rpmalloc_statistics_file(void) { return tmpfile(); }
int64_t binding_rpmalloc_statistics_file_size(void *file) {
  if (fflush(file) || fseek(file, 0, SEEK_END))
    return -1;
  return ftell(file);
}
int32_t binding_rpmalloc_statistics_file_close(void *file) {
  return fclose(file);
}
uint64_t binding_rpmalloc_count(uint32_t index, uint64_t amount) {
  if (index >= 5)
    abort();
  return atomic_fetch_add(&counters[index], amount) + amount;
}
void *binding_rpmalloc_test_map(uint64_t size) {
#if defined(_WIN32)
  return _aligned_malloc(size, 262144);
#else
  void *memory = NULL;
  if (posix_memalign(&memory, 262144, size))
    return NULL;
  return memory;
#endif
}
void binding_rpmalloc_test_unmap(void *memory) {
#if defined(_WIN32)
  _aligned_free(memory);
#else
  free(memory);
#endif
}
typedef struct Worker {
  void (*run)(void);
} Worker;
uint64_t binding_rpmalloc_native_thread(void) {
#if defined(_WIN32)
  return GetCurrentThreadId();
#else
  return (uintptr_t)pthread_self();
#endif
}
#if defined(_WIN32)
static DWORD WINAPI start_worker(void *data) {
  ((Worker *)data)->run();
  return 0;
}
#else
static void *start_worker(void *data) {
  ((Worker *)data)->run();
  return NULL;
}
#endif
int32_t binding_rpmalloc_parallel(void (*run)(void)) {
  Worker worker = {run};
  int count = 0;
  int result = 0;
#if defined(_WIN32)
  HANDLE threads[4];
  for (; count < 4; ++count) {
    threads[count] = CreateThread(NULL, 0, start_worker, &worker, 0, NULL);
    if (!threads[count]) {
      result = 1;
      break;
    }
  }
  for (int i = 0; i < count; ++i) {
    if (WaitForSingleObject(threads[i], INFINITE) != WAIT_OBJECT_0)
      abort();
    if (!CloseHandle(threads[i]))
      result = 1;
  }
#else
  pthread_t threads[4];
  for (; count < 4; ++count)
    if (pthread_create(&threads[count], NULL, start_worker, &worker)) {
      result = 1;
      break;
    }
  for (int i = 0; i < count; ++i)
    if (pthread_join(threads[i], NULL))
      abort();
#endif
  return result;
}
