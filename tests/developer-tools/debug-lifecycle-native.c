// Original SDK oracle: actual process signal actions, no replacement Debug code.
#define _GNU_SOURCE
#include <signal.h>
#include <stdint.h>
#include <string.h>
#include <pthread.h>

static const int signals[5] = {SIGSEGV, SIGTRAP, SIGFPE, SIGBUS, SIGILL};
static struct sigaction original[5];
static struct sigaction baseline[5];
static void prior_handler(int number) { (void)number; }

int32_t original_debug_install_prior(void) {
  for (int i = 0; i < 5; ++i) {
    struct sigaction action;
    memset(&action, 0, sizeof(action));
    action.sa_handler = prior_handler;
    action.sa_flags = SA_RESTART;
    if (sigemptyset(&action.sa_mask) || sigaddset(&action.sa_mask, SIGUSR1) ||
        sigaction(signals[i], &action, &original[i]) ||
        sigaction(signals[i], NULL, &baseline[i])) return 0;
  }
  return 1;
}

int32_t original_debug_changed_actions(void) {
  int count = 0;
  for (int i = 0; i < 5; ++i) {
    struct sigaction current;
    if (sigaction(signals[i], NULL, &current)) return -1;
    if (current.sa_handler != baseline[i].sa_handler) ++count;
  }
  return count;
}

int32_t original_debug_prior_restored(void) {
  for (int i = 0; i < 5; ++i) {
    struct sigaction current;
    if (sigaction(signals[i], NULL, &current)) return 0;
    if (current.sa_handler != baseline[i].sa_handler ||
        current.sa_flags != baseline[i].sa_flags) return 0;
    for (int signal_number = 1; signal_number < NSIG; ++signal_number)
      if (sigismember(&current.sa_mask, signal_number) !=
          sigismember(&baseline[i].sa_mask, signal_number)) return 0;
  }
  return 1;
}

int32_t original_debug_restore_original(void) {
  for (int i = 0; i < 5; ++i)
    if (sigaction(signals[i], &original[i], NULL)) return 0;
  return 1;
}

int32_t original_debug_raise_trap(void) { return raise(SIGTRAP); }

typedef void (*Work)(int64_t);
typedef struct { Work work; int64_t tag; } Work_Argument;
static void *run_work(void *opaque) {
  Work_Argument *argument = opaque;
  argument->work(argument->tag);
  return NULL;
}
int32_t original_debug_run_threads(Work work) {
  if (!work) return 0;
  pthread_t threads[8];
  Work_Argument arguments[8];
  int started = 0, success = 1;
  for (int i = 0; i < 8; ++i) {
    arguments[i].work = work;
    arguments[i].tag = i + 1;
    if (pthread_create(&threads[i], NULL, run_work, &arguments[i]) != 0) {
      success = 0; break;
    }
    ++started;
  }
  for (int i = 0; i < started; ++i)
    if (pthread_join(threads[i], NULL) != 0) success = 0;
  return success;
}
