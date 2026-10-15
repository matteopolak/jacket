#ifndef INDEPENDENT_STRING_MACROS_H
#define INDEPENDENT_STRING_MACROS_H
#define STRING_SIMPLE "value"
#define STRING_ESCAPED "a\"b\\c\n\t"
#define STRING_JOINED "left" "right"
#define STRING_ALIAS STRING_JOINED
#define STRING_UTF8 "caf\xc3\xa9"
#define STRING_EMPTY ""
int string_macro_matches(const char *data, unsigned long long count, int which);
#endif
