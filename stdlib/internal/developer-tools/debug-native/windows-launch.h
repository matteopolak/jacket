#include "windows-command.h"

static wchar_t *debug_wide_text(const unsigned char *text, int64_t count) {
  if (count < 0 || count > INT_MAX || (!text && count)) return NULL;
  int chars = count ? MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
                                          (const char *)text, (int)count, NULL, 0) : 0;
  if (count && !chars) { windows_error("dialog UTF-16 conversion"); return NULL; }
  WCHAR *wide = malloc(((size_t)chars + 1) * sizeof(*wide));
  if (!wide) { fprintf(stderr, "Debug: dialog text allocation failed\n"); return NULL; }
  if (chars && MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, (const char *)text,
                                    (int)count, wide, chars) != chars) {
    windows_error("dialog UTF-16 conversion"); free(wide); return NULL;
  }
  wide[chars] = 0;
  return wide;
}

EXPORT int32_t developer_debug_dialog(const unsigned char *text, int64_t count) {
  WCHAR *wide = debug_wide_text(text, count);
  if (!wide) return 2;
  int response = MessageBoxW(NULL, wide, L"Assertion failed",
                              MB_ABORTRETRYIGNORE | MB_ICONERROR | MB_TASKMODAL);
  free(wide);
  if (response == IDIGNORE) return 0;
  if (response == IDRETRY) return 1;
  if (!response) windows_error("assertion MessageBoxW");
  return 2;
}

static WCHAR *registry_debugger(bool *automatic) {
  HKEY key;
  LSTATUS status = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
      L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\AeDebug", 0,
      KEY_QUERY_VALUE | KEY_WOW64_64KEY, &key);
  if (status != ERROR_SUCCESS) {
    if (status != ERROR_FILE_NOT_FOUND)
      fprintf(stderr, "Debug: AeDebug registry open failed (%ld)\n", (long)status);
    return NULL;
  }
  WCHAR auto_value[16] = {0};
  DWORD type = 0, bytes = sizeof(auto_value);
  status = RegQueryValueExW(key, L"Auto", NULL, &type, (BYTE *)auto_value, &bytes);
  if (status == ERROR_SUCCESS && type == REG_SZ && bytes == 2 * sizeof(WCHAR) &&
      auto_value[0] == L'1' && auto_value[1] == 0)
    *automatic = true;
  bytes = 0;
  status = RegQueryValueExW(key, L"Debugger", NULL, &type, NULL, &bytes);
  WCHAR *command = NULL;
  if (status == ERROR_SUCCESS && (type == REG_SZ || type == REG_EXPAND_SZ) &&
      bytes >= 2 * sizeof(WCHAR) && bytes <= 32768 * sizeof(WCHAR) && bytes % sizeof(WCHAR) == 0) {
    command = malloc(bytes);
    if (!command) fprintf(stderr, "Debug: registry debugger allocation failed\n");
    else {
      DWORD capacity = bytes;
      status = RegQueryValueExW(key, L"Debugger", NULL, &type, (BYTE *)command, &bytes);
      if (status != ERROR_SUCCESS || bytes > capacity || bytes < sizeof(WCHAR) ||
          bytes % sizeof(WCHAR) != 0 || command[bytes / sizeof(WCHAR) - 1] != 0 ||
          wcslen(command) + 1 != bytes / sizeof(WCHAR) ||
          (type != REG_SZ && type != REG_EXPAND_SZ)) {
        free(command); command = NULL;
      } else if (type == REG_EXPAND_SZ) {
        DWORD count = ExpandEnvironmentStringsW(command, NULL, 0);
        WCHAR *expanded = count && count <= 32768 ? malloc((size_t)count * sizeof(WCHAR)) : NULL;
        if (!expanded || ExpandEnvironmentStringsW(command, expanded, count) != count) {
          fprintf(stderr, "Debug: debugger environment expansion failed\n");
          free(expanded); expanded = NULL;
        }
        free(command); command = expanded;
      }
    }
  }
  status = RegCloseKey(key);
  if (status != ERROR_SUCCESS) fprintf(stderr, "Debug: AeDebug registry close failed (%ld)\n", (long)status);
  return command;
}

EXPORT int32_t developer_debug_attach(bool graphics) {
  if (IsDebuggerPresent()) return 1;
  SECURITY_ATTRIBUTES security = {sizeof(security), NULL, TRUE};
  HANDLE event = CreateEventW(&security, TRUE, FALSE, NULL);
  if (!event) { windows_error("debugger CreateEventW"); return 0; }
  bool automatic = false;
  WCHAR *registered = registry_debugger(&automatic);
  WCHAR *command = registered ? debug_format_command(registered, GetCurrentProcessId(), (uintptr_t)event) : NULL;
  free(registered);
  if (!command) {
    WCHAR system[32768];
    UINT count = GetSystemDirectoryW(system, 32768);
    if (!count) windows_error("GetSystemDirectoryW");
    else if (count >= 32768) fprintf(stderr, "Debug: system directory exceeds command limit\n");
    command = malloc(32768 * sizeof(WCHAR));
    if (!command) fprintf(stderr, "Debug: fallback debugger command allocation failed\n");
    int length = command && count && count < 32768
        ? swprintf(command, 32768, L"\"%ls\\vsjitdebugger.exe\" -p %lu", system, (unsigned long)GetCurrentProcessId()) : -1;
    if (length < 0 || length >= 32768) {
      if (command && count && count < 32768) fprintf(stderr, "Debug: fallback debugger command exceeds limit\n");
      free(command); command = NULL;
    }
  }
  if (!command) { if (!CloseHandle(event)) windows_error("debugger event close"); return 0; }
  if (graphics && !automatic) {
    int response = MessageBoxW(NULL, L"Attach a debugger to this process?", L"Debug",
                                MB_YESNO | MB_ICONQUESTION | MB_TASKMODAL);
    if (response != IDYES) {
      if (!response) windows_error("debugger confirmation MessageBoxW");
      free(command); if (!CloseHandle(event)) windows_error("debugger event close"); return 0;
    }
  }
  STARTUPINFOW startup;
  PROCESS_INFORMATION process;
  memset(&startup, 0, sizeof(startup)); startup.cb = sizeof(startup);
  memset(&process, 0, sizeof(process));
  BOOL launched = CreateProcessW(NULL, command, NULL, NULL, TRUE, 0, NULL, NULL, &startup, &process);
  free(command);
  if (!launched) {
    windows_error("debugger CreateProcessW");
    if (!CloseHandle(event)) windows_error("debugger event close");
    return 0;
  }
  if (!CloseHandle(process.hThread)) windows_error("debugger thread close");
  DWORD wait = WaitForSingleObject(process.hProcess, INFINITE), status = 1;
  BOOL selected = wait == WAIT_OBJECT_0 && GetExitCodeProcess(process.hProcess, &status);
  if (!selected) windows_error("debugger wait/exit status");
  if (!CloseHandle(process.hProcess)) windows_error("debugger process close");
  if (!CloseHandle(event)) windows_error("debugger event close");
  if (!selected || status != 0) return 0;
  ULONGLONG start = GetTickCount64();
  do {
    if (IsDebuggerPresent()) return 1;
    Sleep(25);
  } while (GetTickCount64() - start < 60000);
  return 0;
}
