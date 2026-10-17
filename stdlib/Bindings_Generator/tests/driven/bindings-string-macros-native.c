#include "bindings-string-macros.h"
#include <string.h>
int string_macro_matches(const char *data, unsigned long long count, int which) {
  const char *expected;
  size_t length;
  switch (which) {
  case 0: expected = STRING_SIMPLE; length = sizeof(STRING_SIMPLE) - 1; break;
  case 1: expected = STRING_ESCAPED; length = sizeof(STRING_ESCAPED) - 1; break;
  case 2: expected = STRING_JOINED; length = sizeof(STRING_JOINED) - 1; break;
  case 3: expected = STRING_ALIAS; length = sizeof(STRING_ALIAS) - 1; break;
  case 4: expected = STRING_UTF8; length = sizeof(STRING_UTF8) - 1; break;
  case 5: expected = STRING_EMPTY; length = sizeof(STRING_EMPTY) - 1; break;
  default: return 0;
  }
  return count == length && (length == 0 || memcmp(data, expected, length) == 0);
}
