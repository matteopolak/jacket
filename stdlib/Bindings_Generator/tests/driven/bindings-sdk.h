#include <stdint.h>
struct Sized {
  int32_t x;
  uint64_t y;
};
int64_t sized_sum(struct Sized *value);
