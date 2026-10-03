/* Included after the canonical symbol lock/error helper. No separate crash
   thread is promised. Automatic crash-time writes fail if DbgHelp is busy. */
static _Atomic(int32_t) dump_state = 1;
static Trace_String empty_dump;
static _Atomic(Trace_String *) latest_dump = &empty_dump;
static volatile LONG dump_sequence;

EXPORT int32_t developer_debug_set_dump_state(int32_t state) {
  if (state < 0 || state > 2) return 0;
  atomic_store(&dump_state, state);
  return 1;
}

EXPORT Trace_String *developer_debug_latest_dump(void) {
  return atomic_load(&latest_dump);
}

static Trace_String *write_dump(EXCEPTION_POINTERS *exception, bool full,
                                bool automatic) {
  WCHAR *executable = malloc(32768 * sizeof(WCHAR));
  WCHAR *filename = malloc(32768 * sizeof(WCHAR));
  if (!executable || !filename) {
    fprintf(stderr, "Debug: dump filename allocation failed\n");
    free(executable); free(filename); return NULL;
  }
  DWORD length = GetModuleFileNameW(NULL, executable, 32768);
  if (!length || length >= 32768) {
    windows_error("GetModuleFileNameW");
    free(executable); free(filename); return NULL;
  }
  WCHAR *base = wcsrchr(executable, L'\\');
  WCHAR *slash = wcsrchr(executable, L'/');
  if (slash && (!base || slash > base)) base = slash;
  base = base ? base + 1 : executable;
  SYSTEMTIME utc;
  GetSystemTime(&utc);
  int written = swprintf(filename, 32768,
                         L"%ls-%04u%02u%02uT%02u%02u%02u%03uZ-%lu-%lu.dmp",
                         base, utc.wYear, utc.wMonth, utc.wDay, utc.wHour,
                         utc.wMinute, utc.wSecond, utc.wMilliseconds,
                         (unsigned long)GetCurrentProcessId(),
                         (unsigned long)(DWORD)InterlockedIncrement(&dump_sequence));
  free(executable);
  if (written < 0 || written >= 32768) {
    fprintf(stderr, "Debug: dump filename exceeds Windows limit\n");
    free(filename); return NULL;
  }
  int bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, filename,
                                  -1, NULL, 0, NULL, NULL);
  if (!bytes) { windows_error("dump filename UTF-8 conversion"); free(filename); return NULL; }
  Trace_String *result = malloc(sizeof(*result) + (size_t)bytes);
  if (!result) { fprintf(stderr, "Debug: dump result allocation failed\n"); free(filename); return NULL; }
  result->data = (unsigned char *)(result + 1);
  result->count = bytes - 1;
  if (WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, filename, -1,
                          (char *)result->data, bytes, NULL, NULL) != bytes) {
    windows_error("dump filename UTF-8 conversion"); free(filename); free(result); return NULL;
  }
  if (automatic) {
    if (!TryAcquireSRWLockExclusive(&symbol_lock)) {
      fprintf(stderr, "Debug: crash-time DbgHelp is busy; dump not written\n");
      free(filename); free(result); return NULL;
    }
  } else AcquireSRWLockExclusive(&symbol_lock);
  HANDLE file = CreateFileW(filename, GENERIC_WRITE, FILE_SHARE_READ, NULL,
                             CREATE_NEW, FILE_ATTRIBUTE_NORMAL, NULL);
  if (file == INVALID_HANDLE_VALUE) {
    windows_error("minidump file creation");
    ReleaseSRWLockExclusive(&symbol_lock);
    free(filename); free(result); return NULL;
  }
  MINIDUMP_EXCEPTION_INFORMATION information = {GetCurrentThreadId(), exception, FALSE};
  BOOL success = MiniDumpWriteDump(GetCurrentProcess(), GetCurrentProcessId(), file,
                                   full ? MiniDumpWithFullMemory : MiniDumpNormal,
                                   exception ? &information : NULL, NULL, NULL);
  if (!success) windows_error("MiniDumpWriteDump");
  if (success && !FlushFileBuffers(file)) { windows_error("FlushFileBuffers"); success = FALSE; }
  if (!CloseHandle(file)) { windows_error("minidump CloseHandle"); success = FALSE; }
  ReleaseSRWLockExclusive(&symbol_lock);
  if (!success) {
    if (!DeleteFileW(filename)) windows_error("failed minidump removal");
    free(result); result = NULL;
  }
  free(filename);
  return result;
}

EXPORT Trace_String *developer_debug_dump(void *exception, bool full) {
  return write_dump(exception, full, false);
}

static void automatic_dump(EXCEPTION_POINTERS *exception) {
  int32_t state = atomic_load(&dump_state);
  Trace_String *result = state ? write_dump(exception, state == 2, true) : NULL;
  // Automatic results are immutable module-owned storage until process exit.
  // Keeping earlier results alive also makes concurrent borrowed getters safe.
  atomic_store(&latest_dump, result ? result : &empty_dump);
}
