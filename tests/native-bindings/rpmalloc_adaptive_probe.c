/* The runner prefixes every SDK export. This independent SDK instance is
 * never initialized; only its actual private type/atomic definitions are used
 * to read the live canonical adapter's native heap. */
#include "rpmalloc.c"
#include <assert.h>
_Static_assert(sizeof(span_use_t) == 8, "reviewed statistics-off accounting");
int32_t native_current(void *native, uint32_t index) {
  assert(native && index < LARGE_CLASS_COUNT);
  return atomic_load32(&((heap_t *)native)->span_use[index].current);
}
int32_t native_high(void *native, uint32_t index) {
  assert(native && index < LARGE_CLASS_COUNT);
  return atomic_load32(&((heap_t *)native)->span_use[index].high);
}
