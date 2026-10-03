/* The native lifecycle lock protects publication and retirement. Immutable
   context snapshots remain alive while any asynchronous callback is using one. */
#include <stddef.h>
typedef void (*Crash_Callback)(Trace_String *, int64_t, void *);
typedef struct Debug_Crash_State {
  struct Debug_Crash_State *next;
  Crash_Callback callback;
  int64_t context_count;
  max_align_t alignment;
  unsigned char context[];
} Debug_Crash_State;
static _Atomic(Debug_Crash_State *) debug_crash_current;
static _Atomic(uint32_t) debug_crash_active;
static Debug_Crash_State *debug_crash_retired;

static int debug_crash_valid(const void *data, int64_t count) {
  return count >= 0 && (data || !count) &&
         (uint64_t)count <= SIZE_MAX - sizeof(Debug_Crash_State) &&
         atomic_is_lock_free(&debug_crash_current) && atomic_is_lock_free(&debug_crash_active);
}

static Debug_Crash_State *debug_crash_prepare(Crash_Callback callback,
                                             const void *data, int64_t count) {
  if (!debug_crash_valid(data, count)) return NULL;
  Debug_Crash_State *state = malloc(sizeof(*state) + (size_t)count);
  if (!state) { fprintf(stderr, "Debug: crash context allocation failed\n"); return NULL; }
  state->next = NULL;
  state->callback = callback;
  state->context_count = count;
  if (count) memcpy(state->context, data, (size_t)count);
  return state;
}

static void debug_crash_publish(Debug_Crash_State *state) {
  Debug_Crash_State *old = atomic_exchange(&debug_crash_current, state);
  if (old) { old->next = debug_crash_retired; debug_crash_retired = old; }
  // Sequential consistency pairs the reader increment before its pointer load
  // with this exchange followed by the counter check: no old reader is missed.
  if (!atomic_load(&debug_crash_active)) {
    while (debug_crash_retired) {
      old = debug_crash_retired;
      debug_crash_retired = old->next;
      free(old);
    }
  }
}

static int debug_crash_deliver(Trace_String *trace, int64_t count) {
  atomic_fetch_add(&debug_crash_active, 1);
  Debug_Crash_State *state = atomic_load(&debug_crash_current);
  int delivered = state && state->callback;
  if (delivered)
    state->callback(trace, count, state->context_count ? state->context : NULL);
  atomic_fetch_sub(&debug_crash_active, 1);
  return delivered;
}
