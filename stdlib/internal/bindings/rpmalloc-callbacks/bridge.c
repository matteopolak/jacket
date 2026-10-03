/* Independent adapter around the pinned public rpmalloc SDK. Public offsets
 * never enter the SDK's narrower span header: they live in owned map records.
 */
#include <errno.h>
#include <limits.h>
#include <rpmalloc.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "../rpmalloc-heaps/runtime-callback-state.h"
#define STATE (BINDING_JOIN(BINDING_RPMALLOC_FACADE_PREFIX, callback_storage)())
#define cb_config (STATE->config)
#define cb_maps (STATE->maps)
#define cb_map_lock (STATE->map_lock)
#define cb_page_name (STATE->page_name)
#define cb_huge_page_name (STATE->huge_page_name)
#define cb_active (STATE->active)

static void lock_maps(void) {
  while (
      atomic_flag_test_and_set_explicit(&cb_map_lock, memory_order_acquire)) {
  }
}
static void unlock_maps(void) {
  atomic_flag_clear_explicit(&cb_map_lock, memory_order_release);
}

static void *map_memory(size_t size, size_t *offset) {
#if !BINDING_RPMALLOC_RAW
  if (size > INT64_MAX)
    return NULL;
#endif
  BindingMap *record = malloc(sizeof(*record));
  if (!record)
    return NULL;
  size_t original_offset = 0;
  void *address = cb_config.memory_map(size, &original_offset);
  if (!address) {
    free(record);
    return NULL;
  }
  size_t span = rpmalloc_config()->span_size;
  if (!span || (uintptr_t)address % span || original_offset > 65535) {
    cb_config.memory_unmap(address, size, original_offset, size);
    free(record);
    return NULL;
  }
  record->address = address;
  record->size = size;
  record->offset = original_offset;
  lock_maps();
  record->next = cb_maps;
  cb_maps = record;
  unlock_maps();
  *offset = 0;
  return address;
}

static void unmap_memory(void *address, size_t size, size_t offset,
                         size_t release) {
  (void)offset;
  lock_maps();
  BindingMap **link = &cb_maps;
  while (*link &&
         !((uintptr_t)address >= (uintptr_t)(*link)->address &&
           (uintptr_t)address - (uintptr_t)(*link)->address < (*link)->size))
    link = &(*link)->next;
  BindingMap *record = *link;
  if (!record) {
    unlock_maps();
    if (cb_config.error_callback)
      cb_config.error_callback("rpmalloc adapter: unmap outside owned mapping");
    abort();
  }
  size_t original_offset = release ? record->offset : 0;
  void *original_address = release ? record->address : address;
  if (release)
    *link = record->next;
  unlock_maps();
  cb_config.memory_unmap(original_address, size, original_offset, release);
  if (release)
    free(record);
}

static char *copy_name(const void *bytes, size_t length) {
  if (!length)
    return NULL;
  if (!bytes || length == SIZE_MAX)
    return NULL;
  char *copy = malloc(length + 1);
  if (copy) {
    memcpy(copy, bytes, length);
    copy[length] = 0;
  }
  return copy;
}

int32_t binding_rpmalloc_initialize(const rpmalloc_config_t *config,
                                    const void *name, uint64_t name_length,
                                    const void *huge_name,
                                    uint64_t huge_name_length) {
  if (cb_active)
    return EINVAL; /* Duplicate low-level initialization of this owned domain.
                    */
  cb_config = *config;
  cb_page_name = copy_name(name, name_length);
  if (name_length && !cb_page_name)
    return ENOMEM;
  cb_huge_page_name = copy_name(huge_name, huge_name_length);
  if (huge_name_length && !cb_huge_page_name) {
    free(cb_page_name);
    cb_page_name = NULL;
    return ENOMEM;
  }
  rpmalloc_config_t native = cb_config;
  native.page_name = cb_page_name;
  native.huge_page_name = cb_huge_page_name;
  if (cb_config.memory_map) {
    native.memory_map = map_memory;
    native.memory_unmap = unmap_memory;
  }
  int32_t result = rpmalloc_initialize_config(&native);
  if (!result && !rpmalloc_is_thread_initialized())
    result = ENOMEM;
  if (result) {
    rpmalloc_finalize();
    if (cb_maps)
      abort();
    free(cb_page_name);
    free(cb_huge_page_name);
    cb_page_name = cb_huge_page_name = NULL;
  }
  if (!result)
    cb_active = 1;
  return result;
}

void binding_rpmalloc_finalize(void) {
  rpmalloc_finalize();
  cb_active = 0;
  if (cb_maps)
    abort(); /* Do not hide an owned-mapping lifetime leak. */
  free(cb_page_name);
  free(cb_huge_page_name);
  cb_page_name = cb_huge_page_name = NULL;
  memset(&cb_config, 0, sizeof(cb_config));
}
