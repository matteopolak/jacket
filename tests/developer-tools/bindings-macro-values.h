#ifndef DEVELOPER_TOOLS_MACRO_VALUES_H
#define DEVELOPER_TOOLS_MACRO_VALUES_H
#define ORIGINAL_INTEGER 17
#define ORIGINAL_UNSIGNED 0xffffffffffffffffULL
#define ORIGINAL_NEGATIVE (-37)
#define ORIGINAL_REAL (18.5f)
#define ORIGINAL_EXPRESSION (ORIGINAL_INTEGER * 3 + 4)
#define ORIGINAL_LONG_BYTES sizeof(long)
long macro_native_sum(void);
long macro_native_mode(int mode);
unsigned long long macro_native_unsigned(unsigned long long value);
#endif
