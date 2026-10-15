/* Separate SDK introspection object; libc/thread probes do not depend on an
 * allocator domain. */
#include <rpmalloc.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

int32_t binding_rpmalloc_test_names(void) {
  const rpmalloc_config_t *config = rpmalloc_config();
  return config->page_name && config->huge_page_name &&
         !strcmp(config->page_name, "binding-page") &&
         !strcmp(config->huge_page_name, "binding-huge");
}
uint64_t binding_rpmalloc_test_config(uint32_t index) {
  const rpmalloc_config_t *config = rpmalloc_config();
  if (index == 0)
    return config->page_size;
  if (index == 1)
    return config->span_size;
  if (index == 2)
    return config->span_map_count;
  abort();
}
int32_t binding_rpmalloc_test_error(void) {
  const rpmalloc_config_t *config = rpmalloc_config();
  if (!config->error_callback)
    return 0;
  config->error_callback("adapter test error");
  return 1;
}
