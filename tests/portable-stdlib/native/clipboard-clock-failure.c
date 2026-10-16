// Test-only clock injection in this translation unit. No library override,
// Display substitute or clipboard operation. Production queue/cleanup code is
// exercised with real pthreads, pipes and heap allocations.
#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <pthread.h>
#include <stdatomic.h>
#include <time.h>
static _Atomic int waiting_callers;
static int tracked_wait(pthread_cond_t *condition, pthread_mutex_t *mutex) {
  atomic_fetch_add(&waiting_callers, 1);
  return pthread_cond_wait(condition, mutex);
}
static int injected_failure, clock_calls;
static int fail_clock(clockid_t clock, struct timespec *value) {
  (void)clock;
  ++clock_calls;
  if (!injected_failure)
    return clock_gettime(clock, value);
  errno = EIO;
  return -1;
}
#define clock_gettime fail_clock
#define pthread_cond_wait tracked_wait
#include "../../../stdlib/internal/clipboard/native/x11.c"
#undef clock_gettime
#undef pthread_cond_wait
#include <assert.h>

static void *caller(void *argument) {
  Command *command = argument;
  command->status = submit(command);
  return NULL;
}

int main(void) {
  alarm(5);
  Service service = {0};
  double value = 123;
  assert(now(&service, &value) && value >= 0);
  assert(expire_transfers(&service, None));
  injected_failure = 1;
  value = 123;
  assert(!now(&service, &value));
  assert(service.failure == -EIO && value == 123);
  int calls = clock_calls;
  assert(!now(&service, &value) && clock_calls == calls);
  Service expiry = {0}, waiting = {0}, sending = {0};
  assert(!expire_transfers(&expiry, None) && expiry.failure == -EIO);
  XEvent event;
  assert(!wait_event(&waiting, &event, 999) && waiting.failure == -EIO);
  sending.transfers = calloc(1, sizeof *sending.transfers);
  assert(sending.transfers);
  sending.transfers->window = 1;
  sending.transfers->property = 2;
  event.type = PropertyNotify;
  event.xproperty.state = PropertyDelete;
  event.xproperty.window = 1;
  event.xproperty.atom = 2;
  dispatch(&sending, &event);
  assert(sending.failure == -EIO && sending.transfers->offset == 0);
  finish_service(&sending);
  service_error = 0; // Isolated next queue test; no production recovery/retry.
  calls = clock_calls;
  service.data = malloc(7);
  service.transfers = calloc(1, sizeof *service.transfers);
  assert(service.data && service.transfers);
  service.transfers->data = malloc(9);
  assert(service.transfers->data);
  assert(!expire_transfers(&service, None));
  assert(service.transfers); // Clock failure must not pretend expiry happened.
  assert(wait_event(&service, &event, 999) == 0);
  Command read = {.operation = 2};
  assert(receive(&service, &read) == -EIO);
  Command write = {.operation = 1};
  assert(replace(&service, &write) == -EIO);
  assert(!pipe(wakeup));
  ready = 1;
  Command first = {0}, second = {0};
  pthread_t a, b;
  assert(!pthread_create(&a, NULL, caller, &first));
  assert(!pthread_create(&b, NULL, caller, &second));
  int queued = 0;
  for (int attempt = 0; attempt < 1000 && !queued; ++attempt) {
    pthread_mutex_lock(&queue);
    queued = pending != NULL && atomic_load(&waiting_callers) >= 2;
    pthread_mutex_unlock(&queue);
    if (!queued) {
      struct timespec pause = {0, 1000000};
      nanosleep(&pause, NULL);
    }
  }
  assert(queued);
  finish_service(&service);
  assert(!pthread_join(a, NULL) && !pthread_join(b, NULL));
  assert(first.status == -EIO && second.status == -EIO);
  assert(!service.data && !service.transfers && !pending && ready == -1);
  assert(service_error == -EIO);
  assert(initialize() == -EIO && clock_calls == calls);
  unsigned char *text;
  size_t size;
  assert(portable_clipboard_read(0, &text, &size) == -EIO);
  assert(!text && !size);
  assert(portable_clipboard_text(0, (const unsigned char *)"x", 1) == -EIO);
  const unsigned char pixel[] = {1, 2, 3};
  assert(portable_clipboard_bitmap(0, 1, 1, pixel, 3, 0) == -EIO);
  assert(clock_calls == calls);
  close(wakeup[0]);
  close(wakeup[1]);
  return 0;
}
