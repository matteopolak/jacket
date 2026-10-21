/* Independent host operations; no parser or canonical generator state. */
#define _GNU_SOURCE
/* The CRT marks fopen-family and getenv deprecated; their secure forms add nothing here. */
#define _CRT_SECURE_NO_WARNINGS
#define _DARWIN_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <windows.h>
#else
#include <dlfcn.h>
#include <unistd.h>
#endif

/* A Windows DLL exports only what is marked; elsewhere every non-static function is visible. */
#if defined(_WIN32)
#define GP_EXPORT __declspec(dllexport)
#else
#define GP_EXPORT
#endif

#if defined(_WIN32)
static wchar_t *gp_wide_path(const char *path) {
    int count = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, NULL, 0);
    if (count <= 0) return NULL;
    wchar_t *wide = malloc((size_t)count * sizeof *wide);
    if (!wide) return NULL;
    if (!MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, wide, count)) {
        free(wide);
        return NULL;
    }
    return wide;
}
#endif

GP_EXPORT int gp_write_temp(const void *data, uint64_t size, char *path, uint64_t capacity) {
    if (!path || capacity < 32 || (size && !data) || size > SIZE_MAX) return 0;
#if defined(_WIN32)
    wchar_t directory[MAX_PATH], wide_path[MAX_PATH];
    DWORD length = GetTempPathW(MAX_PATH, directory);
    if (!length || length >= MAX_PATH) return 0;
    if (!GetTempFileNameW(directory, L"jai", 0, wide_path)) return 0;
    int needed = WideCharToMultiByte(CP_UTF8, 0, wide_path, -1, NULL, 0, NULL, NULL);
    if (needed <= 0 || (uint64_t)needed > capacity ||
        !WideCharToMultiByte(CP_UTF8, 0, wide_path, -1, path, needed, NULL, NULL)) {
        DeleteFileW(wide_path);
        return 0;
    }
    FILE *file = _wfopen(wide_path, L"wb");
#else
    const char *directory = getenv("TMPDIR");
    if (!directory || !*directory) directory = "/tmp";
    int length = snprintf(path, (size_t)capacity, "%s/jai-symbols-XXXXXX", directory);
    if (length < 0 || (uint64_t)length >= capacity) return 0;
    int fd = mkstemp(path);
    if (fd < 0) return 0;
    FILE *file = fdopen(fd, "wb");
    if (!file) close(fd);
#endif
    if (!file) {
#if defined(_WIN32)
        DeleteFileW(wide_path);
#else
        remove(path);
#endif
        return 0;
    }
    int ok = !size || fwrite(data, 1, (size_t)size, file) == (size_t)size;
    if (fclose(file)) ok = 0;
    if (!ok) {
#if defined(_WIN32)
        DeleteFileW(wide_path);
#else
        remove(path);
#endif
    }
    return ok;
}

GP_EXPORT void *gp_library_open(const char *path) {
#if defined(_WIN32)
    wchar_t *wide = gp_wide_path(path);
    if (!wide) return NULL;
    HMODULE handle = LoadLibraryW(wide);
    free(wide);
    return (void *)handle;
#else
    return dlopen(path, RTLD_NOW | RTLD_LOCAL);
#endif
}
GP_EXPORT int gp_library_symbol(void *handle, const char *name) {
    if (!handle || !name) return 0;
#if defined(_WIN32)
    return GetProcAddress((HMODULE)handle, name) != NULL;
#else
    dlerror();
    (void)dlsym(handle, name);
    return dlerror() == NULL;
#endif
}
GP_EXPORT void gp_library_close(void *handle) {
    if (!handle) return;
#if defined(_WIN32)
    FreeLibrary((HMODULE)handle);
#else
    dlclose(handle);
#endif
}
GP_EXPORT const char *gp_environment(const char *name) {
    return getenv(name);
}
