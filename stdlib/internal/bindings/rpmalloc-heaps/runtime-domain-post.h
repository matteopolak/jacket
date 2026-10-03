#if !PLATFORM_WINDOWS
#include <pthread.h>
#endif

struct BindingThreadSlot {
  BindingDomain *domain;
  heap_t *heap;
  unsigned call_depth;
  uint64_t generation;
  BindingThreadSlot *next;
  BindingThreadSlot *redirect;
};
typedef struct BindingOperationScope BindingOperationScope;
struct BindingOperationScope {
  BindingThreadSlot slot;
  BindingThreadSlot *previous_slot;
  BindingDomain *previous_domain;
  BindingOperationScope *previous_scope;
  unsigned borrowed;
};
_Static_assert(sizeof(BindingOperationScope) == 80 &&
                   offsetof(BindingOperationScope, borrowed) == 72,
               "Private operation scope protocol layout");
static _Thread_local BindingOperationScope *binding_current_scope;
static _Thread_local BindingThreadSlot *binding_thread_slots;
static _Thread_local BindingThreadSlot *binding_slot_override;
static _Thread_local heap_t *binding_default_thread_heap;
static _Thread_local unsigned binding_default_depth;
static void binding_domain_unref(BindingDomain *domain) {
  if (atomic_fetch_sub_explicit(&domain->references, 1, memory_order_acq_rel) !=
      1)
    return;
  free(domain->sdk_memory_heaps);
  binding_metadata_destroy(domain);
#if ENABLE_GLOBAL_CACHE
  free(domain->owned_cache_storage);
#endif
  free(domain);
}
#if !PLATFORM_WINDOWS
static pthread_key_t binding_slot_key;
static atomic_flag binding_slot_key_lock = ATOMIC_FLAG_INIT;
static unsigned binding_slot_key_ready;
static void binding_thread_slots_destroy(void *value) {
  BindingThreadSlot *slot = value;
  binding_thread_slots = NULL;
  while (slot) {
    BindingThreadSlot *next = slot->next;
    binding_domain_unref(slot->domain);
    free(slot);
    slot = next;
  }
}
static int binding_thread_key_create(void) {
  while (atomic_flag_test_and_set_explicit(&binding_slot_key_lock,
                                           memory_order_acquire)) {
  }
  int result = 0;
  if (!binding_slot_key_ready) {
    result =
        pthread_key_create(&binding_slot_key, binding_thread_slots_destroy);
    if (!result)
      binding_slot_key_ready = 1;
  }
  atomic_flag_clear_explicit(&binding_slot_key_lock, memory_order_release);
  return result;
}
#endif
static BindingThreadSlot *binding_find_thread_slot(BindingDomain *domain) {
  if (binding_slot_override && binding_slot_override->domain == domain) {
    BindingThreadSlot *slot = binding_slot_override;
    while (slot->redirect)
      slot = slot->redirect;
    return slot;
  }
#if PLATFORM_WINDOWS
  if (domain->owned_fls_ready) {
    BindingThreadSlot *attached = FlsGetValue(domain->sdk_fls_key);
    if (attached)
      return attached;
  }
#endif
  for (BindingThreadSlot *slot = binding_thread_slots; slot; slot = slot->next)
    if (slot->domain == domain) {
      if (slot->generation != domain->generation) {
        slot->heap = NULL;
        slot->generation = domain->generation;
      }
      return slot;
    }
  return NULL;
}
static BindingThreadSlot *binding_thread_slot(BindingDomain *domain) {
  BindingThreadSlot *existing = binding_find_thread_slot(domain);
  if (existing)
    return existing;
#if !PLATFORM_WINDOWS
  if (binding_thread_key_create())
    return NULL;
#endif
  BindingThreadSlot *slot = calloc(1, sizeof(*slot));
  if (!slot)
    return NULL;
  slot->domain = domain;
  slot->generation = domain->generation;
  slot->next = binding_thread_slots;
#if !PLATFORM_WINDOWS
  if (pthread_setspecific(binding_slot_key, slot)) {
    free(slot);
    return NULL;
  }
#endif
  atomic_fetch_add_explicit(&domain->references, 1, memory_order_relaxed);
  binding_thread_slots = slot;
  return slot;
}
static BindingThreadSlot *binding_required_thread_slot(void) {
  BindingThreadSlot *slot = binding_find_thread_slot(binding_selected_domain);
  if (!slot)
    abort(); // Internal API misuse: operations need a scope or prepared slot.
  return slot;
}
static void binding_drop_empty_thread_slot(BindingDomain *domain) {
  BindingThreadSlot **link = &binding_thread_slots;
  while (*link && (*link)->domain != domain)
    link = &(*link)->next;
  BindingThreadSlot *slot = *link;
  if (slot == binding_slot_override && !binding_current_scope)
    return;
  if (!slot || slot->call_depth ||
      (slot->heap && slot->generation == domain->generation))
    return;
  *link = slot->next;
#if !PLATFORM_WINDOWS
  if (pthread_setspecific(binding_slot_key, binding_thread_slots))
    abort();
#endif
  binding_domain_unref(domain);
  free(slot);
}
static heap_t **binding_domain_heap_slot(void) {
  return binding_selected_domain ? &binding_required_thread_slot()->heap
                                 : &binding_default_thread_heap;
}
static unsigned *binding_domain_depth_slot(void) {
  return binding_selected_domain ? &binding_required_thread_slot()->call_depth
                                 : &binding_default_depth;
}
void *API(domain_create)(void) {
  BindingDomain *domain = calloc(1, sizeof(*domain));
  if (!domain)
    return NULL;
  BindingDomain initial = BINDING_DOMAIN_INITIALIZER;
  *domain = initial;
  if (!binding_thread_slot(domain)) {
    free(domain);
    return NULL;
  }
  return domain;
}
void API(domain_destroy)(BindingDomain *domain) {
  if (!domain)
    return;
  if (domain->sdk_rpmalloc_initialized ||
      (domain->owned_active_calls &&
       (binding_selected_domain != domain || domain->owned_active_calls != 1 ||
        !*binding_domain_depth_slot())))
    abort();
  // Allocator_Proc SHUTDOWN can retire within an outer operation lease. Its
  // thread slot keeps the domain alive until that lease's final leave, while
  // the canonical Jai owner immediately relinquishes its reference.
  domain->closed = 1;
  ++domain->generation;
  binding_drop_empty_thread_slot(domain);
  binding_domain_unref(domain);
}
int32_t API(domain_prepare_thread)(BindingDomain *domain) {
  return domain && !domain->closed && binding_thread_slot(domain) ? 0 : ENOMEM;
}
void *API(select_domain)(BindingDomain *domain) {
  BindingDomain *previous = binding_selected_domain;
  if (domain && domain->closed)
    abort();
  binding_selected_domain = domain;
  return previous;
}
void API(restore_domain)(BindingDomain *previous) {
  binding_selected_domain = previous;
}
BindingCallbackState *API(callback_storage)(void) {
  return &binding_domain_current()->owned_callbacks;
}
void *API(enter)(BindingDomain *domain);
void API(leave)(void *previous);
void API(scope_enter)(BindingDomain *domain, BindingOperationScope *scope) {
  memset(scope, 0, sizeof(*scope));
  scope->previous_domain = binding_selected_domain;
  scope->previous_slot = binding_slot_override;
  scope->previous_scope = binding_current_scope;
  BindingThreadSlot *slot = binding_find_thread_slot(domain);
  if (!slot) {
    slot = &scope->slot;
    slot->domain = domain;
    slot->generation = domain->generation;
    scope->borrowed = 1;
    atomic_fetch_add_explicit(&domain->references, 1, memory_order_relaxed);
  }
  binding_current_scope = scope;
  binding_slot_override = slot;
  (void)API(enter)(domain);
}
void API(scope_leave)(BindingOperationScope *scope) {
  API(leave)(scope->previous_domain);
  binding_slot_override = scope->previous_slot;
  binding_current_scope = scope->previous_scope;
  if (scope->borrowed)
    binding_domain_unref(scope->slot.domain);
}
static int binding_domain_prepare_heap(void) {
  if (!binding_selected_domain)
    return 1;
  BindingThreadSlot *slot = binding_required_thread_slot();
  BindingOperationScope *borrowed = binding_current_scope;
  while (borrowed && (!borrowed->borrowed || slot != &borrowed->slot))
    borrowed = borrowed->previous_scope;
  if (!borrowed)
    return 1;
  BindingThreadSlot *previous = binding_slot_override;
  binding_slot_override = NULL;
  BindingThreadSlot *persistent = binding_thread_slot(binding_selected_domain);
  binding_slot_override = previous;
  if (!persistent) {
    errno = ENOMEM;
    return 0;
  }
  persistent->call_depth = slot->call_depth;
  slot->call_depth = 0;
  slot->redirect = persistent;
  return 1;
}
#if defined(_WIN32) && (!defined(BUILD_DYNAMIC_LINK) || !BUILD_DYNAMIC_LINK)
void *API(enter)(BindingDomain *domain);
void API(leave)(void *previous);
void *API(thread_mutation)(void);
void API(thread_finalized)(void *token);
static DWORD binding_domain_fls_allocate(void) {
  DWORD key =
      FlsAlloc(binding_selected_domain ? &binding_domain_windows_destructor
                                       : &_rpmalloc_thread_destructor);
  binding_domain_current()->owned_fls_ready = key != FLS_OUT_OF_INDEXES;
  return key;
}
static void binding_domain_fls_free(void) {
  if (binding_domain_current()->owned_fls_ready) {
    if (!FlsFree(fls_key))
      abort();
    binding_domain_current()->owned_fls_ready = 0;
  }
}
static void binding_domain_fls_heap(heap_t *heap) {
  if (binding_slot_override && !binding_current_scope) {
    binding_required_thread_slot()->heap = heap;
    return;
  }
  if (!binding_domain_current()->owned_fls_ready) {
    if (heap)
      abort();
    return;
  }
  if (!binding_selected_domain) {
    if (!FlsSetValue(fls_key, heap))
      abort();
    return;
  }
  BindingThreadSlot *slot = binding_required_thread_slot();
  if (!FlsSetValue(fls_key, heap ? slot : NULL))
    abort();
  // An attached slot belongs solely to FLS. Explicit thread finalization
  // returns it to the calling thread until its operation lease is released.
  BindingThreadSlot **link = &binding_thread_slots;
  while (*link && *link != slot)
    link = &(*link)->next;
  if (*link)
    *link = slot->next;
  if (heap) {
    slot->next = NULL;
  } else {
    slot->next = binding_thread_slots;
    binding_thread_slots = slot;
  }
}
static void NTAPI binding_domain_windows_destructor(void *value) {
  BindingThreadSlot *slot = value;
  if (!slot)
    return;
  BindingDomain *previous = binding_selected_domain;
  BindingThreadSlot *previous_slot = binding_slot_override;
  BindingOperationScope *previous_scope = binding_current_scope;
  binding_current_scope = NULL;
  binding_selected_domain = slot->domain;
  binding_slot_override = slot;
  if (!slot->domain->closed && !slot->domain->owned_finalizing &&
      slot->generation == slot->domain->generation) {
    void *previous_operation = API(enter)(slot->domain);
    void *mutation = API(thread_mutation)();
    _rpmalloc_thread_destructor(slot->heap);
    API(thread_finalized)(mutation);
    API(leave)(previous_operation);
  }
  binding_slot_override = previous_slot;
  binding_current_scope = previous_scope;
  binding_selected_domain = previous;
  binding_domain_unref(slot->domain);
  free(slot);
}
#endif
