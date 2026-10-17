// Original native SDK lifecycle test; no public x64 context is involved.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>

__declspec(dllimport) DWORD platform_io_arm64_crash_init(void);
__declspec(dllimport) DWORD platform_io_arm64_crash_fini(void);

static LONG WINAPI sentinel(EXCEPTION_POINTERS *exception) {
    (void)exception;
    return EXCEPTION_CONTINUE_SEARCH;
}

int main(void) {
    LPTOP_LEVEL_EXCEPTION_FILTER original = SetUnhandledExceptionFilter(sentinel);
    UINT mode = GetErrorMode();
    assert(platform_io_arm64_crash_fini() == ERROR_INVALID_STATE);
    for (unsigned cycle = 0; cycle < 3; ++cycle) {
        assert(platform_io_arm64_crash_init() == ERROR_SUCCESS);
        assert(platform_io_arm64_crash_init() == ERROR_SUCCESS);
        LPTOP_LEVEL_EXCEPTION_FILTER installed = SetUnhandledExceptionFilter(sentinel);
        assert(installed != NULL && installed != sentinel);
        assert(SetUnhandledExceptionFilter(installed) == sentinel);
        assert(platform_io_arm64_crash_fini() == ERROR_SUCCESS);
        assert(SetUnhandledExceptionFilter(sentinel) == installed);
        assert(SetUnhandledExceptionFilter(installed) == sentinel);
        assert(platform_io_arm64_crash_fini() == ERROR_SUCCESS);
        assert(SetUnhandledExceptionFilter(sentinel) == sentinel);
        assert(GetErrorMode() == mode);
        assert(platform_io_arm64_crash_fini() == ERROR_INVALID_STATE);
    }
    // An application may replace our filter before final shutdown.
    assert(platform_io_arm64_crash_init() == ERROR_SUCCESS);
    SetUnhandledExceptionFilter(sentinel);
    assert(platform_io_arm64_crash_fini() == ERROR_SUCCESS);
    assert(SetUnhandledExceptionFilter(original) == sentinel);
    return 0;
}
