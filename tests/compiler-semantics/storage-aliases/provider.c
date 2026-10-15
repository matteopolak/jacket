// Original C provider, independent of any system-library implementation.
#include <stdint.h>
#if defined(_WIN32)
#define API __declspec(dllexport)
#else
#define API __attribute__((visibility("default")))
#endif
struct cleanroom_pair {
  uint32_t left, right;
};
API uint64_t cleanroom_alias_counter = 7;
API struct cleanroom_pair cleanroom_alias_pair = {11, 13};
API uint64_t cleanroom_alias_read(void) { return cleanroom_alias_counter; }
API void cleanroom_alias_write(uint64_t value) {
  cleanroom_alias_counter = value;
}
