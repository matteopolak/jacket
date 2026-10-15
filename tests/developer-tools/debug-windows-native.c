#define _WIN32_WINNT 0x0600
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdint.h>
#include <string.h>

typedef struct { int64_t count; unsigned char *data; } Trace_String;
__declspec(dllimport) int64_t developer_debug_backtrace(void **, int64_t, int64_t);
__declspec(dllimport) Trace_String *developer_debug_symbols(void *const *, int64_t);
__declspec(dllimport) void developer_debug_free_symbols(Trace_String *);
__declspec(dllimport) int32_t developer_debug_debugger_present(void);
__declspec(dllimport) int32_t developer_debug_readable(const void *);
__declspec(dllimport) int32_t developer_debug_output(const unsigned char *, int64_t);
__declspec(dllimport) void developer_debug_abort(int32_t);
__declspec(dllimport) int32_t developer_debug_breakpoint(void);

int main(int argc, char **argv) {
  if (argc == 2 && strcmp(argv[1], "abort") == 0) developer_debug_abort(2);
  assert(developer_debug_debugger_present() == (IsDebuggerPresent() != FALSE));
  assert(!developer_debug_readable(NULL));
  unsigned char *page = VirtualAlloc(NULL, 4096, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
  assert(page);
  *page = 42;
  assert(developer_debug_readable(page));
  DWORD old;
  assert(VirtualProtect(page, 4096, PAGE_NOACCESS, &old));
  assert(!developer_debug_readable(page));
  assert(VirtualFree(page, 0, MEM_RELEASE));
  assert(!developer_debug_readable(page));
  void *addresses[32];
  int64_t count = developer_debug_backtrace(addresses, 32, 0);
  assert(count > 0 && count <= 32);
  Trace_String *symbols = developer_debug_symbols(addresses, count);
  assert(symbols);
  for (int64_t i = 0; i < count; ++i) {
    assert(symbols[i].count > 0 && symbols[i].data);
    assert(symbols[i].data[symbols[i].count] == 0);
  }
  developer_debug_free_symbols(symbols);
  assert(developer_debug_output(NULL, 0));
  HANDLE reader, writer;
  assert(CreatePipe(&reader, &writer, NULL, 0));
  HANDLE saved_error = GetStdHandle(STD_ERROR_HANDLE);
  assert(SetStdHandle(STD_ERROR_HANDLE, writer));
  unsigned char expected[] = {65, 0, 0xc3, 0xa9, 10};
  assert(developer_debug_output(expected, sizeof(expected)));
  unsigned char received[sizeof(expected)];
  DWORD read;
  assert(ReadFile(reader, received, sizeof(received), &read, NULL));
  assert(read == sizeof(expected) && memcmp(received, expected, sizeof(expected)) == 0);
  assert(SetStdHandle(STD_ERROR_HANDLE, saved_error));
  assert(CloseHandle(reader) && CloseHandle(writer));
  if (!IsDebuggerPresent()) assert(!developer_debug_breakpoint());
  return 0;
}
