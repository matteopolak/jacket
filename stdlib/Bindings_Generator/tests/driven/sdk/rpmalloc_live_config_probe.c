/* Observe the exact native backend selected by the private test scope. */
#include <rpmalloc.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
int32_t native_error_mode(void) {
  return getenv("RPMALLOC_TEST_CLEAR_ERROR") != NULL;
}
#ifndef LIVE_NAME_COPY
#define LIVE_NAME_COPY binding_rpmalloc_facade_name_copy
#endif
extern char *LIVE_NAME_COPY(uint32_t huge);
int32_t native_name(uint32_t huge, const void *expected, uint64_t length) {
  char *copy = LIVE_NAME_COPY(huge);
  if (!length) {
    int32_t empty = !copy;
    free(copy);
    return empty;
  }
  int32_t equal =
      copy && expected && !memcmp(copy, expected, length) && !copy[length];
  free(copy);
  return equal;
}
int32_t native_error(void) {
  const rpmalloc_config_t *config = rpmalloc_config();
  if (!config->error_callback)
    return 0;
  config->error_callback("live allocator error");
  return 1;
}
uint64_t native_size(uint32_t index) {
  const rpmalloc_config_t *config = rpmalloc_config();
  if (index == 0)
    return config->page_size;
  if (index == 1)
    return config->span_size;
  if (index == 2)
    return config->span_map_count;
  abort();
}
