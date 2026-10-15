#include "bindings-variadic.h"
#include <stdarg.h>
int original_variadic(int initial, ...) {
  va_list arguments;
  va_start(arguments, initial);
  int small = va_arg(arguments, int);
  int promoted_unsigned = va_arg(arguments, int);
  double promoted_float = va_arg(arguments, double);
  int *pointer = va_arg(arguments, int *);
  va_end(arguments);
  return initial + small + promoted_unsigned + (int)(promoted_float * 100) + *pointer;
}
double original_variadic_numbers(int count, ...) {
  va_list arguments;
  va_start(arguments, count);
  double sum = 0;
  for (int i = 0; i < count; ++i) sum += va_arg(arguments, double);
  va_end(arguments);
  return sum;
}
Original_Variadic original_get_variadic(void) { return original_variadic; }
