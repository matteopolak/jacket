// Original SDK session-isolation and private-runtime cleanup regression.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <stdint.h>
#include <assert.h>

__declspec(dllimport) DWORD platform_io_windows_crash_prepare(HANDLE process);
__declspec(dllimport) void platform_io_windows_crash_install(LPTOP_LEVEL_EXCEPTION_FILTER handler);
__declspec(dllimport) DWORD platform_io_windows_crash_finish(void);

__declspec(dllexport) __declspec(noinline) int sdk_identity_leaf(int value) {
    return value + 1;
}

static LONG WINAPI sentinel(EXCEPTION_POINTERS *exception) {
    (void)exception;
    return EXCEPTION_CONTINUE_SEARCH;
}

static LONG WINAPI installed(EXCEPTION_POINTERS *exception) {
    (void)exception;
    return EXCEPTION_CONTINUE_SEARCH;
}

int main(void) {
    HANDLE external_session = GetCurrentProcess();
    assert(SymInitialize(external_session, NULL, TRUE));
    LPTOP_LEVEL_EXCEPTION_FILTER original = SetUnhandledExceptionFilter(sentinel);
    UINT mode = GetErrorMode();
    assert(platform_io_windows_crash_finish() == ERROR_INVALID_STATE);
    for (unsigned cycle = 0; cycle < 3; ++cycle) {
        // Our engine must not collide with another component's pseudo-handle session.
        assert(platform_io_windows_crash_prepare(GetCurrentProcess()) == ERROR_SUCCESS);
        assert(platform_io_windows_crash_prepare(GetCurrentProcess()) == ERROR_SUCCESS);
        platform_io_windows_crash_install(installed);
        platform_io_windows_crash_install(installed);
        assert(SetUnhandledExceptionFilter(installed) == installed);
        assert(platform_io_windows_crash_finish() == ERROR_SUCCESS);
        assert(SetUnhandledExceptionFilter(sentinel) == sentinel);
        assert(GetErrorMode() == mode);
        assert(platform_io_windows_crash_finish() == ERROR_INVALID_STATE);
        _Alignas(SYMBOL_INFO) unsigned char storage[sizeof(SYMBOL_INFO) + 256] = {0};
        SYMBOL_INFO *symbol = (SYMBOL_INFO *)storage;
        symbol->SizeOfStruct = sizeof(SYMBOL_INFO);
        symbol->MaxNameLen = 256;
        DWORD64 displacement = 0;
        assert(SymFromAddr(external_session, (DWORD64)(uintptr_t)sdk_identity_leaf, &displacement, symbol));
        assert(symbol->NameLen != 0);
    }
    assert(platform_io_windows_crash_prepare(GetCurrentProcess()) == ERROR_SUCCESS);
    platform_io_windows_crash_install(installed);
    SetUnhandledExceptionFilter(sentinel);
    assert(platform_io_windows_crash_finish() == ERROR_SUCCESS);
    assert(SetUnhandledExceptionFilter(original) == sentinel);
    assert(SymCleanup(external_session));
    return 0;
}
