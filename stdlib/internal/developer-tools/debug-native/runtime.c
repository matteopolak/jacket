#define _GNU_SOURCE
#include <errno.h>
#include <execinfo.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#ifdef __APPLE__
#include <sys/ucontext.h>
#include <mach/mach.h>
#include <mach/mach_vm.h>
#include <sys/sysctl.h>
#else
#include <ucontext.h>
#include <sys/uio.h>
#endif

typedef struct {
  int64_t count;
  unsigned char *data;
} Trace_String;

int64_t developer_debug_backtrace(void **trace, int64_t maxcount,
                                  int64_t skip) {
  if (!trace || maxcount <= 0 || skip < 0 || skip >= 4096)
    return 0;
  if (maxcount > 4096 - skip - 1)
    maxcount = 4096 - skip - 1;
  int capacity = (int)(maxcount + skip + 1);
  void **frames = malloc((size_t)capacity * sizeof(*frames));
  if (!frames) {
    fprintf(stderr, "Debug: stack capture allocation failed\n");
    return 0;
  }
  int captured = backtrace(frames, capacity);
  int begin = (int)skip + 1; /* Omit this C helper's own frame. */
  int count = captured > begin ? captured - begin : 0;
  if (count)
    memcpy(trace, frames + begin, (size_t)count * sizeof(*trace));
  free(frames);
  return count;
}

Trace_String *developer_debug_symbols(void *const *trace, int64_t count) {
  if (!trace || count <= 0 || count > 4096)
    return NULL;
  char **symbols = backtrace_symbols(trace, (int)count);
  if (!symbols)
    return NULL;
  size_t size = (size_t)count * sizeof(Trace_String);
  for (int64_t i = 0; i < count; ++i) {
    size_t length = strlen(symbols[i]);
    if (length > SIZE_MAX - size - 1) {
      free(symbols);
      return NULL;
    }
    size += length + 1;
  }
  Trace_String *result = malloc(size);
  if (!result) {
    free(symbols);
    return NULL;
  }
  unsigned char *cursor = (unsigned char *)(result + count);
  for (int64_t i = 0; i < count; ++i) {
    size_t length = strlen(symbols[i]);
    result[i].count = (int64_t)length;
    result[i].data = cursor;
    memcpy(cursor, symbols[i], length + 1);
    cursor += length + 1;
  }
  free(symbols);
  return result;
}

void developer_debug_free_symbols(Trace_String *symbols) { free(symbols); }

int32_t developer_debug_debugger_present(void) {
#ifdef __APPLE__
  int mib[4] = {CTL_KERN, KERN_PROC, KERN_PROC_PID, getpid()};
  struct kinfo_proc info = {0};
  size_t count = sizeof(info);
  if (sysctl(mib, 4, &info, &count, NULL, 0) != 0) {
    perror("Debug sysctl");
    return 0;
  }
  return (info.kp_proc.p_flag & P_TRACED) != 0;
#else
  FILE *file = fopen("/proc/self/status", "r");
  if (!file) {
    perror("Debug /proc/self/status");
    return 0;
  }
  char line[512];
  int traced = 0;
  int found = 0;
  while (fgets(line, sizeof(line), file)) {
    long pid = 0;
    if (sscanf(line, "TracerPid: %ld", &pid) == 1) {
      found = 1;
      traced = pid > 0;
      break;
    }
  }
  if (ferror(file))
    perror("Debug status read");
  if (!found)
    fprintf(stderr, "Debug: TracerPid is missing from /proc/self/status\n");
  if (fclose(file) != 0)
    perror("Debug status close");
  return traced;
#endif
}

int32_t developer_debug_readable(const void *pointer) {
  if (!pointer)
    return 0;
  unsigned char byte;
#ifdef __APPLE__
  mach_vm_size_t copied = 0;
  kern_return_t status = mach_vm_read_overwrite(
      mach_task_self(), (mach_vm_address_t)(uintptr_t)pointer, 1,
      (mach_vm_address_t)(uintptr_t)&byte, &copied);
  return status == KERN_SUCCESS && copied == 1;
#else
  struct iovec local = {&byte, 1};
  struct iovec remote = {(void *)pointer, 1};
  return process_vm_readv(getpid(), &local, 1, &remote, 1, 0) == 1;
#endif
}

int32_t developer_debug_output(const unsigned char *text, int64_t count) {
  if (count < 0 || (!text && count))
    return 0;
  while (count) {
    ssize_t written = write(STDERR_FILENO, text, (size_t)count);
    if (written < 0 && errno == EINTR)
      continue;
    if (written <= 0)
      return 0;
    text += written;
    count -= written;
  }
  return 1;
}

void developer_debug_abort(int32_t code) { _Exit(code); }

int32_t developer_debug_breakpoint(void) {
  if (!developer_debug_debugger_present())
    return 0;
  return raise(SIGTRAP) == 0;
}

static pthread_mutex_t once_lock = PTHREAD_MUTEX_INITIALIZER;
static int debug_once_lock(void) { return pthread_mutex_lock(&once_lock); }
static int debug_once_unlock(void) { return pthread_mutex_unlock(&once_lock); }
#define DEBUG_EXPORT
#include "once.h"

/* No allocator, unwinder or symbolizer runs in the fatal signal handler. */
#include "crash-state.h"
static pthread_mutex_t lifecycle_lock = PTHREAD_MUTEX_INITIALIZER;
static const int handled_signals[] = {SIGSEGV, SIGTRAP, SIGFPE, SIGBUS, SIGILL};
static struct sigaction previous_actions[5];
static int action_installed[5];

static void fatal_signal(int signal_number, siginfo_t *info, void *opaque) {
  (void)signal_number;
  (void)info;
  uintptr_t pc = 0;
  ucontext_t *state = opaque;
  if (state) {
#if defined(__APPLE__) && defined(__aarch64__)
    pc = (uintptr_t)__darwin_arm_thread_state64_get_pc(state->uc_mcontext->__ss);
#elif defined(__APPLE__) && defined(__x86_64__)
    pc = (uintptr_t)state->uc_mcontext->__ss.__rip;
#elif defined(__linux__) && defined(__aarch64__)
    pc = (uintptr_t)state->uc_mcontext.pc;
#elif defined(__linux__) && defined(__x86_64__)
    pc = (uintptr_t)state->uc_mcontext.gregs[REG_RIP];
#else
#error Unsupported native signal context: implement the official OS ABI first
#endif
  }
  unsigned char frame[2 + sizeof(uintptr_t) * 2];
  frame[0] = '0';
  frame[1] = 'x';
  for (size_t i = 0; i < sizeof(uintptr_t) * 2; ++i) {
    unsigned shift = (unsigned)((sizeof(uintptr_t) * 2 - 1 - i) * 4);
    frame[2 + i] = (unsigned char)"0123456789abcdef"[(pc >> shift) & 15];
  }
  Trace_String trace = {(int64_t)sizeof(frame), frame};
  if (!debug_crash_deliver(&trace, 1)) {
    (void)developer_debug_output(frame, (int64_t)sizeof(frame));
    (void)developer_debug_output((const unsigned char *)"\n", 1);
  }
  _Exit(4); /* Returning into a faulting instruction is not safe. */
}

int32_t developer_debug_enable_handlers(Crash_Callback callback, const void *data, int64_t count) {
  if (!debug_crash_valid(data, count)) return 0;
  int lock_error = pthread_mutex_lock(&lifecycle_lock);
  if (lock_error) { errno = lock_error; perror("Debug lifecycle lock"); return 0; }
  int complete = 1;
  for (size_t i = 0; i < 5; ++i) if (!action_installed[i]) complete = 0;
  if (complete) {
    lock_error = pthread_mutex_unlock(&lifecycle_lock);
    if (lock_error) { errno = lock_error; perror("Debug lifecycle unlock"); return 0; }
    return 1;
  }
  Debug_Crash_State *state = debug_crash_prepare(callback, data, count);
  if (!state) {
    lock_error = pthread_mutex_unlock(&lifecycle_lock);
    if (lock_error) { errno = lock_error; perror("Debug lifecycle unlock"); }
    return 0;
  }
  int success = 1;
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  action.sa_sigaction = fatal_signal;
  action.sa_flags = SA_SIGINFO;
  if (sigemptyset(&action.sa_mask) != 0) { perror("Debug signal mask"); success = 0; }
  for (size_t i = 0; success && i < 5; ++i)
    if (sigaddset(&action.sa_mask, handled_signals[i]) != 0) {
      perror("Debug signal mask member"); success = 0;
    }
  if (success) debug_crash_publish(state);
  else free(state);
  for (size_t i = 0; success && i < 5; ++i) {
    if (action_installed[i]) continue;
    if (sigaction(handled_signals[i], &action, &previous_actions[i]) != 0) {
      perror("Debug install sigaction"); success = 0;
    } else action_installed[i] = 1;
  }
  if (!success) for (size_t i = 0; i < 5; ++i) if (action_installed[i]) {
    if (sigaction(handled_signals[i], &previous_actions[i], NULL) != 0)
      perror("Debug rollback sigaction");
    else action_installed[i] = 0;
  }
  lock_error = pthread_mutex_unlock(&lifecycle_lock);
  if (lock_error) { errno = lock_error; perror("Debug lifecycle unlock"); success = 0; }
  return success;
}

int32_t developer_debug_disable_handlers(void) {
  int lock_error = pthread_mutex_lock(&lifecycle_lock);
  if (lock_error) { errno = lock_error; perror("Debug lifecycle lock"); return 0; }
  int success = 1;
  for (size_t i = 0; i < 5; ++i) if (action_installed[i]) {
    if (sigaction(handled_signals[i], &previous_actions[i], NULL) != 0) {
      perror("Debug restore sigaction"); success = 0;
    } else action_installed[i] = 0;
  }
  lock_error = pthread_mutex_unlock(&lifecycle_lock);
  if (lock_error) { errno = lock_error; perror("Debug lifecycle unlock"); success = 0; }
  return success;
}
