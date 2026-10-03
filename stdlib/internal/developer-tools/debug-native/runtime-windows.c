#define _WIN32_WINNT 0x0600
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <limits.h>
#include <stdbool.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>

#define EXPORT __declspec(dllexport)
#define SYMBOL_CHARS 1024
#define FRAME_BYTES (SYMBOL_CHARS * 4 + 40)

typedef struct {
  int64_t count;
  unsigned char *data;
} Trace_String;

_Static_assert(sizeof(Trace_String) == 16, "64-bit Jai string ABI");
_Static_assert(sizeof(WCHAR) == 2, "Windows Unicode ABI");
static SRWLOCK symbol_lock = SRWLOCK_INIT;
static HANDLE symbol_process;

static void windows_error(const char *operation) {
  fprintf(stderr, "Debug: %s failed (Windows error %lu)\n", operation,
          (unsigned long)GetLastError());
}

#include "windows-dumps.h"
#include "windows-launch.h"

EXPORT int64_t developer_debug_backtrace(void **trace, int64_t maxcount,
                                         int64_t skip) {
  if (!trace || maxcount <= 0 || skip < 0 || skip >= 4096) return 0;
  if (maxcount > 4096 - skip - 1) maxcount = 4096 - skip - 1;
  return (int64_t)CaptureStackBackTrace((ULONG)skip + 1, (ULONG)maxcount,
                                       trace, NULL);
}

static void cleanup_symbols(void) {
  AcquireSRWLockExclusive(&symbol_lock);
  if (symbol_process) {
    if (!SymCleanup(symbol_process)) windows_error("SymCleanup");
    if (!CloseHandle(symbol_process)) windows_error("CloseHandle");
    symbol_process = NULL;
  }
  ReleaseSRWLockExclusive(&symbol_lock);
}

/* Caller owns symbol_lock. One DbgHelp session is shared by init and capture. */
static int initialize_symbols_locked(void) {
  if (symbol_process) return 1;
  HANDLE process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                               FALSE, GetCurrentProcessId());
  if (!process) { windows_error("OpenProcess"); return 0; }
  WCHAR *path = malloc(32768 * sizeof(*path));
  if (!path) {
    fprintf(stderr, "Debug: symbol search path allocation failed\n");
    if (!CloseHandle(process)) windows_error("CloseHandle");
    return 0;
  }
  DWORD path_size = GetModuleFileNameW(NULL, path, 32768);
  if (!path_size || path_size >= 32768) {
    windows_error("GetModuleFileNameW");
    free(path);
    if (!CloseHandle(process)) windows_error("CloseHandle");
    return 0;
  }
  WCHAR *separator = wcsrchr(path, L'\\');
  if (!separator) {
    fprintf(stderr, "Debug: executable directory is unavailable\n");
    free(path);
    if (!CloseHandle(process)) windows_error("CloseHandle");
    return 0;
  }
  if (separator == path + 2 && path[1] == L':') separator[1] = 0;
  else *separator = 0;
  if (!SymInitializeW(process, path, TRUE)) {
    windows_error("SymInitializeW");
    free(path);
    if (!CloseHandle(process)) windows_error("CloseHandle");
    return 0;
  }
  free(path);
  if (atexit(cleanup_symbols) != 0) {
    fprintf(stderr, "Debug: cannot register DbgHelp cleanup\n");
    if (!SymCleanup(process)) windows_error("SymCleanup");
    if (!CloseHandle(process)) windows_error("CloseHandle");
    return 0;
  }
  symbol_process = process;
  return 1;
}

EXPORT int32_t developer_debug_initialize(void) {
  AcquireSRWLockExclusive(&symbol_lock);
  int result = initialize_symbols_locked();
  ReleaseSRWLockExclusive(&symbol_lock);
  return result;
}

EXPORT Trace_String *developer_debug_symbols(void *const *trace, int64_t count) {
  if (!trace || count <= 0 || count > 4096) return NULL;
  SYMBOL_INFOW *symbol = calloc(1, sizeof(*symbol) + SYMBOL_CHARS * sizeof(WCHAR));
  size_t size = (size_t)count * (sizeof(Trace_String) + FRAME_BYTES);
  Trace_String *result = malloc(size);
  if (!symbol || !result) {
    fprintf(stderr, "Debug: symbol allocation failed\n");
    free(symbol); free(result);
    return NULL;
  }
  AcquireSRWLockExclusive(&symbol_lock);
  bool success = initialize_symbols_locked() != 0;
  unsigned char *text = (unsigned char *)(result + count);
  for (int64_t i = 0; success && i < count; ++i) {
    symbol->SizeOfStruct = sizeof(*symbol);
    symbol->MaxNameLen = SYMBOL_CHARS;
    DWORD64 displacement = 0;
    unsigned char *row = text + (size_t)i * FRAME_BYTES;
    int length = 0;
    if (SymFromAddrW(symbol_process, (DWORD64)(uintptr_t)trace[i], &displacement, symbol)) {
      if (symbol->NameLen >= SYMBOL_CHARS) {
        fprintf(stderr, "Debug: symbol exceeds configured name limit\n");
        success = false;
        break;
      }
      if (symbol->NameLen) {
        length = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, symbol->Name,
                                     (int)symbol->NameLen, (char *)row,
                                     FRAME_BYTES - 40, NULL, NULL);
        if (!length) { windows_error("WideCharToMultiByte"); success = false; break; }
      } else {
        length = snprintf((char *)row, FRAME_BYTES, "0x%llx",
                          (unsigned long long)(uintptr_t)trace[i]);
        if (length < 0 || length >= FRAME_BYTES) { success = false; break; }
        result[i].data = row;
        result[i].count = length;
        continue;
      }
      int suffix = snprintf((char *)row + length, FRAME_BYTES - (size_t)length,
                            "+0x%llx", (unsigned long long)displacement);
      if (suffix < 0 || suffix >= FRAME_BYTES - length) { success = false; break; }
      length += suffix;
    } else {
      DWORD error = GetLastError();
      if (error != ERROR_INVALID_ADDRESS && error != ERROR_NOT_FOUND &&
          error != ERROR_MOD_NOT_FOUND && error != ERROR_SUCCESS) {
        windows_error("SymFromAddrW"); success = false; break;
      }
      length = snprintf((char *)row, FRAME_BYTES, "0x%llx",
                        (unsigned long long)(uintptr_t)trace[i]);
      if (length < 0 || length >= FRAME_BYTES) { success = false; break; }
    }
    result[i].data = row;
    result[i].count = length;
  }
  ReleaseSRWLockExclusive(&symbol_lock);
  free(symbol);
  if (!success) { free(result); return NULL; }
  return result;
}

EXPORT void developer_debug_free_symbols(Trace_String *symbols) { free(symbols); }

EXPORT int32_t developer_debug_debugger_present(void) {
  return IsDebuggerPresent() != FALSE;
}

EXPORT int32_t developer_debug_readable(const void *pointer) {
  if (!pointer) return 0;
  unsigned char byte;
  SIZE_T copied = 0;
  return ReadProcessMemory(GetCurrentProcess(), pointer, &byte, 1, &copied) && copied == 1;
}

EXPORT int32_t developer_debug_output(const unsigned char *text, int64_t count) {
  if (count < 0 || (!text && count)) return 0;
  if (!count) return 1;
  HANDLE handle = GetStdHandle(STD_ERROR_HANDLE);
  if (!handle || handle == INVALID_HANDLE_VALUE) return 0;
  DWORD mode;
  if (GetConsoleMode(handle, &mode)) {
    if (count > INT_MAX) return 0;
    int chars = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, (const char *)text,
                                   (int)count, NULL, 0);
    if (!chars) return 0;
    WCHAR *wide = malloc((size_t)chars * sizeof(*wide));
    if (!wide) return 0;
    if (MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, (const char *)text,
                           (int)count, wide, chars) != chars) { free(wide); return 0; }
    DWORD position = 0;
    while (position < (DWORD)chars) {
      DWORD written = 0;
      if (!WriteConsoleW(handle, wide + position, (DWORD)chars - position,
                         &written, NULL) || !written) { free(wide); return 0; }
      position += written;
    }
    free(wide);
    return 1;
  }
  while (count) {
    DWORD chunk = count > MAXDWORD ? MAXDWORD : (DWORD)count;
    DWORD written = 0;
    if (!WriteFile(handle, text, chunk, &written, NULL) || !written) return 0;
    text += written;
    count -= written;
  }
  return 1;
}

EXPORT void developer_debug_abort(int32_t code) { _Exit(code); }

EXPORT int32_t developer_debug_breakpoint(void) {
  if (!IsDebuggerPresent()) return 0;
  DebugBreak();
  return 1;
}

static SRWLOCK once_lock = SRWLOCK_INIT;
static int debug_once_lock(void) { AcquireSRWLockExclusive(&once_lock); return 0; }
static int debug_once_unlock(void) { ReleaseSRWLockExclusive(&once_lock); return 0; }
#define DEBUG_EXPORT EXPORT
#include "once.h"

#include "crash-state.h"
static SRWLOCK lifecycle_lock = SRWLOCK_INIT;
static LPTOP_LEVEL_EXCEPTION_FILTER previous_filter;
static bool filter_installed;

static LONG WINAPI fatal_exception(EXCEPTION_POINTERS *exception) {
  automatic_dump(exception);
  uintptr_t pc = exception && exception->ExceptionRecord
                     ? (uintptr_t)exception->ExceptionRecord->ExceptionAddress : 0;
  unsigned char frame[2 + sizeof(uintptr_t) * 2];
  frame[0] = '0'; frame[1] = 'x';
  for (size_t i = 0; i < sizeof(uintptr_t) * 2; ++i) {
    unsigned shift = (unsigned)((sizeof(uintptr_t) * 2 - 1 - i) * 4);
    frame[2 + i] = (unsigned char)"0123456789abcdef"[(pc >> shift) & 15];
  }
  Trace_String trace = {(int64_t)sizeof(frame), frame};
  if (!debug_crash_deliver(&trace, 1)) {
    (void)developer_debug_output(frame, (int64_t)sizeof(frame));
    (void)developer_debug_output((const unsigned char *)"\n", 1);
  }
  _Exit(4);
}

EXPORT int32_t developer_debug_enable_handlers(Crash_Callback callback, const void *data, int64_t count) {
  if (!debug_crash_valid(data, count)) return 0;
  AcquireSRWLockExclusive(&lifecycle_lock);
  if (!filter_installed) {
    Debug_Crash_State *state = debug_crash_prepare(callback, data, count);
    if (!state) { ReleaseSRWLockExclusive(&lifecycle_lock); return 0; }
    debug_crash_publish(state);
    previous_filter = SetUnhandledExceptionFilter(fatal_exception);
    filter_installed = true;
  }
  ReleaseSRWLockExclusive(&lifecycle_lock);
  return 1;
}

EXPORT int32_t developer_debug_disable_handlers(void) {
  AcquireSRWLockExclusive(&lifecycle_lock);
  if (filter_installed) {
    (void)SetUnhandledExceptionFilter(previous_filter);
    filter_installed = false;
  }
  ReleaseSRWLockExclusive(&lifecycle_lock);
  return 1;
}
