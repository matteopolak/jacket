/* Original test-only SDK helper: cause a hardware access violation without
 * relying on Jai lowering, near-null guards, or a software-raised exception. */
#include <windows.h>

__declspec(dllexport) DWORD platform_io_native_fault(void) {
    volatile unsigned char *page = VirtualAlloc(NULL, 1,
        MEM_RESERVE | MEM_COMMIT, PAGE_NOACCESS);
    if (page == NULL) return GetLastError();
    *page = 23;
    /* Unreachable on a conforming OS. Do not hide setup/cleanup failures if the
     * write returns; the Jai caller reports it and the runner fails. */
    if (!VirtualFree((void *)page, 0, MEM_RELEASE)) return GetLastError();
    return ERROR_SUCCESS;
}
