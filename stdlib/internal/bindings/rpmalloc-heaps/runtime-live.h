/* Fixed SDK thunks read the live public record. Names are copied at use,
 * never shared temporary C strings whose replacement could race a reader. */
#if !PLATFORM_WINDOWS
#include <sys/mman.h>
#endif

int32_t API(configure_live)(void (*provider)(int32_t, const void **,
                                             uint64_t *)) {
  if (_rpmalloc_initialized)
    return EINVAL;
  binding_name_provider = provider;
  return 0;
}
static char *binding_rpmalloc_live_name(int huge) {
  const void *bytes = NULL;
  uint64_t length = 0;
  if (binding_name_provider)
    binding_name_provider(huge, &bytes, &length);
  else {
    bytes = huge ? _memory_config.huge_page_name : _memory_config.page_name;
    if (bytes)
      length = strlen(bytes);
  }
  if (!length)
    return NULL;
  if (!bytes || length >= SIZE_MAX) {
    errno = ENOMEM;
    rpmalloc_assert(0, "rpmalloc facade: invalid live name storage");
    return NULL;
  }
  char *copy = malloc((size_t)length + 1);
  if (!copy) {
    rpmalloc_assert(0, "rpmalloc facade: live name allocation failed");
    return NULL;
  }
  memcpy(copy, bytes, (size_t)length);
  copy[length] = 0;
  return copy;
}
char *API(name_copy)(uint32_t huge) {
  return binding_rpmalloc_live_name(!!huge);
}
void *API(default_map)(size_t size, size_t *offset) {
  return _rpmalloc_mmap_os(size, offset);
}
void API(default_unmap)(void *address, size_t size, size_t offset,
                        size_t release) {
#if !PLATFORM_WINDOWS
  if (!release && !binding_disable_unmap) {
    if (madvise(address, size, retained_advice ? MADV_FREE : MADV_DONTNEED))
      rpmalloc_assert(0, "Failed to advise retained allocator pages");
    return;
  }
#endif
  _rpmalloc_unmap_os(address, size, offset, release);
}
void API(default_error)(const char *message) {
  (void)message;
#if ENABLE_ASSERTS
  assert(0 && "rpmalloc assertion failed without an error callback");
#endif
}
