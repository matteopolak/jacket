// Original C provider, independent of any system-library implementation.
#include <stdint.h>
#if defined(_WIN32)
#define API __declspec(dllexport)
#else
#define API __attribute__((visibility("default")))
#endif
struct alias_pair_data {
  uint32_t left, right;
};
API uint64_t storage_alias_counter = 7;
API struct alias_pair_data storage_alias_pair = {11, 13};
API uint64_t storage_alias_read(void) { return storage_alias_counter; }
API void storage_alias_write(uint64_t value) {
  storage_alias_counter = value;
}
