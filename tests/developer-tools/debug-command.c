// Original bounds/substitution oracle. This runs on Mac as well as Windows.
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>
#include "../../stdlib/internal/developer-tools/debug-native/windows-command.h"
int main(void) {
  wchar_t *command = debug_format_command(L"\"C:\\SDK debugger.exe\" -p %ld -e %ld %%", 123, 456);
  assert(command && wcscmp(command, L"\"C:\\SDK debugger.exe\" -p 123 -e 456 %") == 0);
  free(command);
  command = debug_format_command(L"debug -p %ld", 0xffffffffU, UINTPTR_MAX);
  assert(command && wcsstr(command, L"4294967295"));
  free(command);
  assert(!debug_format_command(L"debug", 1, 2));
  assert(!debug_format_command(L"debug %n", 1, 2));
  assert(!debug_format_command(L"debug %ld %ld %ld", 1, 2));
  assert(!debug_format_command(L"debug %", 1, 2));
  wchar_t *long_command = malloc(32769 * sizeof(wchar_t));
  assert(long_command);
  for (int i = 0; i < 32768; ++i) long_command[i] = L'a';
  long_command[32768] = 0;
  assert(!debug_format_command(long_command, 1, 2));
  free(long_command);
  return 0;
}
