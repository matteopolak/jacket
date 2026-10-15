#ifndef ORIGINAL_VARIADIC_H
#define ORIGINAL_VARIADIC_H
int original_variadic(int varargs_0, ...);
double original_variadic_numbers(int count, ...);
typedef int (*Original_Variadic)(int initial, ...);
Original_Variadic original_get_variadic(void);
#endif
