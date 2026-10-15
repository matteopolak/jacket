// Original SDK oracle: real minidump files and explicit/automatic separation.
#define _WIN32_WINNT 0x0600
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>
typedef struct { int64_t count; unsigned char *data; } Trace_String;
typedef void (*Crash_Callback)(Trace_String *, int64_t, void *);
__declspec(dllimport) Trace_String *developer_debug_dump(void *, bool);
__declspec(dllimport) Trace_String *developer_debug_latest_dump(void);
__declspec(dllimport) int32_t developer_debug_set_dump_state(int32_t);
__declspec(dllimport) void developer_debug_free_symbols(Trace_String *);
__declspec(dllimport) int32_t developer_debug_enable_handlers(Crash_Callback, const void *, int64_t);
__declspec(dllimport) int32_t developer_debug_dialog(const unsigned char *, int64_t);

static int expected_state;
static void validate_dump(Trace_String *name, int full) {
  assert(name && name->count > 0 && name->count < 32768);
  assert(name->data[name->count] == 0 && strstr((char *)name->data, ".dmp"));
  WCHAR path[32768];
  assert(MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, (char *)name->data,
                             (int)name->count + 1, path, 32768));
  HANDLE file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
  assert(file != INVALID_HANDLE_VALUE);
  MINIDUMP_HEADER header;
  DWORD read;
  assert(ReadFile(file, &header, sizeof(header), &read, NULL) && read == sizeof(header));
  assert(memcmp(&header.Signature, "MDMP", 4) == 0);
  assert(((header.Flags & MiniDumpWithFullMemory) != 0) == (full != 0));
  assert(CloseHandle(file));
  assert(DeleteFileW(path));
}

static void automatic_callback(Trace_String *trace, int64_t count, void *data) {
  assert(trace && count == 1 && !data);
  Trace_String *latest = developer_debug_latest_dump();
  if (!expected_state) assert(latest && latest->count == 0);
  else validate_dump(latest, expected_state == 2);
  fputs("original automatic dump callback\n", stderr);
}

int main(int argc, char **argv) {
  assert(developer_debug_latest_dump()->count == 0);
  if (argc == 2) {
    expected_state = strcmp(argv[1], "default") == 0 ? 1 : atoi(argv[1]);
    if (strcmp(argv[1], "default") != 0) assert(developer_debug_set_dump_state(expected_state));
    assert(developer_debug_enable_handlers(automatic_callback, NULL, 0));
    RaiseException(0xe1234567, EXCEPTION_NONCONTINUABLE, 0, NULL);
    return 1;
  }
  assert(!developer_debug_set_dump_state(-1));
  assert(!developer_debug_set_dump_state(3));
  assert(developer_debug_set_dump_state(0));
  Trace_String *mini = developer_debug_dump(NULL, false);
  validate_dump(mini, 0);
  assert(developer_debug_latest_dump()->count == 0);
  developer_debug_free_symbols(mini);
  Trace_String *full = developer_debug_dump(NULL, true);
  validate_dump(full, 1);
  assert(developer_debug_latest_dump()->count == 0);
  developer_debug_free_symbols(full);
  return 0;
}
