/* A synchronous native call borrows its caller's context only on that thread.
 * Restoring the previous pointer supports nesting and prevents stale contexts
 * from escaping into later SDK callbacks or native thread destructors. */
static _Thread_local void *active_context;

void *binding_callback_context_enter(void *context) {
  void *previous = active_context;
  active_context = context;
  return previous;
}

void binding_callback_context_leave(void *previous) {
  active_context = previous;
}

void *binding_callback_context_current(void) { return active_context; }
