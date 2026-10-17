#include "bindings-macro-values.h"
long macro_native_sum(void) { return ORIGINAL_INTEGER + ORIGINAL_EXPRESSION; }
long macro_native_mode(int mode) { return mode + 2; }
unsigned long long macro_native_unsigned(unsigned long long value) { return value - 1; }
