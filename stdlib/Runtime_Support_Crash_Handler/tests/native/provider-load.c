/* Independent Windows SDK loader check; no crash-handler functions are called.
 */
#define WIN32_LEAN_AND_MEAN
#include <stdio.h>
#include <windows.h>

static int file_identity(const wchar_t *path,
                         BY_HANDLE_FILE_INFORMATION *info) {
  HANDLE file =
      CreateFileW(path, FILE_READ_ATTRIBUTES,
                  FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL,
                  OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
  if (file == INVALID_HANDLE_VALUE)
    return 0;
  BOOL ok = GetFileInformationByHandle(file, info);
  BOOL closed = CloseHandle(file);
  return ok && closed;
}

int wmain(int argc, wchar_t **argv) {
  if (argc != 3) {
    fputs("usage: provider-load EXPECTED_DLL EXPORT\n", stderr);
    return 2;
  }
  wchar_t loaded[32768];
  char export_name[256];
  if (!WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, argv[2], -1,
                           export_name, sizeof(export_name), NULL, NULL))
    return 3;
  /* Same ordinary loader search used by linked consumer executables. */
  HMODULE provider = LoadLibraryW(L"platform_io_windows_crash.dll");
  if (!provider) {
    fprintf(stderr, "provider LoadLibraryW failed: %lu\n", GetLastError());
    return 4;
  }
  if (!GetProcAddress(provider, export_name)) {
    fprintf(stderr, "provider export %s missing: %lu\n", export_name,
            GetLastError());
    return 5;
  }
  DWORD length = GetModuleFileNameW(provider, loaded, 32768);
  if (!length || length >= 32768)
    return 6;
  BY_HANDLE_FILE_INFORMATION actual_info, expected_info;
  if (!file_identity(loaded, &actual_info) ||
      !file_identity(argv[1], &expected_info))
    return 7;
  if (actual_info.dwVolumeSerialNumber != expected_info.dwVolumeSerialNumber ||
      actual_info.nFileIndexHigh != expected_info.nFileIndexHigh ||
      actual_info.nFileIndexLow != expected_info.nFileIndexLow) {
    fputs("ordinary loader selected a different provider file\n", stderr);
    return 8;
  }
  if (!FreeLibrary(provider))
    return 9;
  puts("matching source-built runtime provider loaded through normal DLL "
       "search");
  return 0;
}
