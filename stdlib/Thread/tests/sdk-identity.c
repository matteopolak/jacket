// Independent POSIX public-header ABI and creation/lifetime control.
#define _GNU_SOURCE 1
#include <pthread.h>
#include <semaphore.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#if defined(__linux__) && defined(__aarch64__)
_Static_assert(sizeof(pthread_t) == 8, "pthread identity storage");
_Static_assert(sizeof(pthread_mutex_t) == 48, "ARM64 mutex storage");
_Static_assert(_Alignof(pthread_mutex_t) == 8, "ARM64 mutex alignment");
_Static_assert(sizeof(pthread_mutexattr_t) == 8, "ARM64 mutex attributes");
_Static_assert(_Alignof(pthread_mutexattr_t) == 4, "ARM64 attribute alignment");
#endif

size_t thread_sdk_size(unsigned kind) {
  switch (kind) {
  case 0:
    return sizeof(pthread_t);
  case 1:
    return sizeof(pthread_mutex_t);
  case 2:
    return sizeof(pthread_mutexattr_t);
  case 3:
    return sizeof(pthread_cond_t);
  case 4:
    return sizeof(sem_t);
  default:
    return 0;
  }
}

size_t thread_sdk_alignment(unsigned kind) {
  switch (kind) {
  case 0:
    return _Alignof(pthread_t);
  case 1:
    return _Alignof(pthread_mutex_t);
  case 2:
    return _Alignof(pthread_mutexattr_t);
  case 3:
    return _Alignof(pthread_cond_t);
  case 4:
    return _Alignof(sem_t);
  default:
    return 0;
  }
}

int thread_sdk_mutex_init(void *storage) {
  return pthread_mutex_init(storage, NULL);
}

int thread_sdk_mutex_destroy(void *storage) {
  return pthread_mutex_destroy(storage);
}

struct gate {
  pthread_mutex_t mutex;
  pthread_cond_t changed;
  pthread_t identity;
  int released;
  int identity_matches;
  int naming_status;
};

static void *entry(void *argument) {
  struct gate *g = argument;
  if (pthread_mutex_lock(&g->mutex))
    return (void *)(uintptr_t)1;
  while (!g->released) {
    if (pthread_cond_wait(&g->changed, &g->mutex)) {
      (void)pthread_mutex_unlock(&g->mutex);
      return (void *)(uintptr_t)2;
    }
  }
  g->identity_matches = pthread_equal(g->identity, pthread_self());
#if defined(__APPLE__)
  g->naming_status = pthread_setname_np("sdk-identity");
#else
  g->naming_status = pthread_setname_np(pthread_self(), "sdk-identity");
#endif
  if (pthread_mutex_unlock(&g->mutex))
    return (void *)(uintptr_t)3;
  return (void *)(uintptr_t)71;
}

// Native C control only: no interpreter scheduler identities cross this API.
int thread_sdk_creation_control(void) {
  struct gate g = {0};
  pthread_mutexattr_t attr;
  int code = pthread_mutexattr_init(&attr);
  if (code)
    return code;
  code = pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
  if (!code)
    code = pthread_mutex_init(&g.mutex, &attr);
  int cleanup = pthread_mutexattr_destroy(&attr);
  if (code)
    return code;
  if (cleanup) {
    (void)pthread_mutex_destroy(&g.mutex);
    return cleanup;
  }
  code = pthread_cond_init(&g.changed, NULL);
  if (code) {
    (void)pthread_mutex_destroy(&g.mutex);
    return code;
  }
  code = pthread_mutex_lock(&g.mutex);
  if (code)
    goto cleanup;
  code = pthread_create(&g.identity, NULL, entry, &g);
  if (code) {
    (void)pthread_mutex_unlock(&g.mutex);
    goto cleanup;
  }
  g.released = 1;
  cleanup = pthread_cond_signal(&g.changed);
  if (cleanup && !code)
    code = cleanup;
  cleanup = pthread_mutex_unlock(&g.mutex);
  if (cleanup && !code)
    code = cleanup;
  void *result = NULL;
  cleanup = pthread_join(g.identity, &result);
  if (cleanup)
    abort(); // Do not return live thread storage on an unjoinable thread.
  if (!code && (result != (void *)(uintptr_t)71 || !g.identity_matches))
    code = 100;
  if (!code)
    code = g.naming_status;
cleanup:
  cleanup = pthread_cond_destroy(&g.changed);
  if (cleanup && !code)
    code = cleanup;
  cleanup = pthread_mutex_destroy(&g.mutex);
  if (cleanup && !code)
    code = cleanup;
  return code;
}

#ifdef THREAD_SDK_MAIN
int main(void) { return thread_sdk_creation_control(); }
#endif
