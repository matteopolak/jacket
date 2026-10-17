// Independent native exception diagnostics against the public Windows SDK.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include "stack-iteration.h"

#if !defined(_M_X64) && !defined(__x86_64__)
#error The Windows crash handler supports x64 only
#endif
_Static_assert(sizeof(CONTEXT) == 1232, "public x64 context ABI");
_Static_assert(offsetof(CONTEXT, Rsp) == 152, "public stack pointer offset");
_Static_assert(offsetof(CONTEXT, Rbp) == 160, "public frame pointer offset");
_Static_assert(offsetof(CONTEXT, Rip) == 248, "public instruction pointer offset");
_Static_assert(sizeof(STACKFRAME64) == 264, "public stack frame ABI");
_Static_assert(sizeof(EXCEPTION_RECORD) == 152, "public exception ABI");

static DWORD initialization;
static LONG reporting;
static HANDLE requested_process;
static HANDLE symbol_process;
static LPTOP_LEVEL_EXCEPTION_FILTER installed_filter;
static LPTOP_LEVEL_EXCEPTION_FILTER previous_filter;
static UINT previous_error_mode;
static SRWLOCK lifecycle_lock = SRWLOCK_INIT;
// DbgHelp is single-threaded. Normal calls wait; a fault must never deadlock
// if it interrupted a thread already inside the symbol engine.
static SRWLOCK symbol_lock = SRWLOCK_INIT;

static void write_bytes(const char *data, DWORD count) {
    HANDLE output = GetStdHandle(STD_ERROR_HANDLE);
    while (count != 0) {
        DWORD written = 0;
        if (!WriteFile(output, data, count, &written, NULL) || written == 0) {
            // Diagnostics cannot continue if stderr itself has failed.
            if (!TerminateProcess(GetCurrentProcess(), ERROR_WRITE_FAULT)) ExitProcess(ERROR_WRITE_FAULT);
            ExitProcess(ERROR_WRITE_FAULT);
        }
        data += written;
        count -= written;
    }
}

#define WRITE_TEXT(text) write_bytes((text), (DWORD)(sizeof(text) - 1))

static void write_hex(uint64_t value) {
    char text[18] = {'0', 'x'};
    static const char digits[] = "0123456789abcdef";
    for (unsigned i = 0; i < 16; ++i) {
        text[17 - i] = digits[value & 15];
        value >>= 4;
    }
    write_bytes(text, sizeof(text));
}

static void write_frame(HANDLE process, DWORD64 address) {
    WRITE_TEXT("  ");
    write_hex(address);
    _Alignas(SYMBOL_INFO) unsigned char storage[sizeof(SYMBOL_INFO) + 1024] = {0};
    SYMBOL_INFO *symbol = (SYMBOL_INFO *)storage;
    symbol->SizeOfStruct = sizeof(SYMBOL_INFO);
    symbol->MaxNameLen = 1024;
    DWORD64 displacement = 0;
    if (SymFromAddr(process, address, &displacement, symbol)) {
        WRITE_TEXT(" ");
        DWORD length = symbol->NameLen;
        if (length > 1023) length = 1023;
        write_bytes(symbol->Name, length);
        WRITE_TEXT(" + ");
        write_hex(displacement);
    }
    WRITE_TEXT("\n");
}

typedef struct {
    CONTEXT context;
    STACKFRAME64 frame;
    HANDLE process;
    HANDLE thread;
} native_walk;

static int advance_frame(void *data, platform_stack_frame *frame) {
    native_walk *walk = data;
    if (!StackWalk64(IMAGE_FILE_MACHINE_AMD64, walk->process, walk->thread,
        &walk->frame, &walk->context, NULL, SymFunctionTableAccess64, SymGetModuleBase64, NULL)) return 0;
    frame->pc = walk->frame.AddrPC.Offset;
    frame->sp = walk->frame.AddrStack.Offset;
    return 1;
}

static void emit_frame(void *data, platform_stack_frame frame) {
    native_walk *walk = data;
    write_frame(walk->process, frame.pc);
}

static void walk_stack(const void *source, int skips, HANDLE process) {
    if (source == NULL) return;
    native_walk walk = {0};
    // Public byte-shaped contexts need not have the SDK's alignment. Copy into
    // aligned native storage; never modify or reinterpret a foreign record.
    memcpy(&walk.context, source, sizeof(walk.context));
    walk.frame.AddrPC.Offset = walk.context.Rip;
    walk.frame.AddrStack.Offset = walk.context.Rsp;
    walk.frame.AddrFrame.Offset = walk.context.Rbp;
    walk.frame.AddrPC.Mode = walk.frame.AddrStack.Mode = walk.frame.AddrFrame.Mode = AddrModeFlat;
    walk.process = process;
    walk.thread = GetCurrentThread();
    platform_stack_frame first = {walk.context.Rip, walk.context.Rsp};
    enum platform_stack_result result = platform_stack_iterate(first, skips, &walk, advance_frame, emit_frame);
    if (result == PLATFORM_STACK_STALLED) WRITE_TEXT("native unwind made no progress\n");
    if (result == PLATFORM_STACK_LIMIT) WRITE_TEXT("native stack truncated at 64 frames\n");
}

__declspec(dllexport) void platform_io_windows_crash_backtrace(const void *source, int skips, HANDLE process) {
    AcquireSRWLockExclusive(&symbol_lock);
    if (initialization == 2 && process == requested_process)
        walk_stack(source, skips, symbol_process);
    else
        WRITE_TEXT("symbol engine is not initialized\n");
    ReleaseSRWLockExclusive(&symbol_lock);
}

__declspec(dllexport) LONG platform_io_windows_crash_handle(EXCEPTION_POINTERS *exception, void *snapshot, HANDLE process) {
    if (exception == NULL || exception->ExceptionRecord == NULL || exception->ContextRecord == NULL)
        return EXCEPTION_CONTINUE_SEARCH;
    if (InterlockedCompareExchange(&reporting, 1, 0) != 0) return EXCEPTION_CONTINUE_SEARCH;
    if (snapshot != NULL) memcpy(snapshot, exception->ContextRecord, sizeof(CONTEXT));
    WRITE_TEXT("Windows exception ");
    write_hex(exception->ExceptionRecord->ExceptionCode);
    WRITE_TEXT(" at ");
    write_hex((uintptr_t)exception->ExceptionRecord->ExceptionAddress);
    WRITE_TEXT("\nnative stack:\n");
    if (TryAcquireSRWLockExclusive(&symbol_lock)) {
        if (initialization == 2 && process == requested_process)
            walk_stack(exception->ContextRecord, 0, symbol_process);
        else
            WRITE_TEXT("symbol engine is not initialized\n");
        ReleaseSRWLockExclusive(&symbol_lock);
    } else {
        WRITE_TEXT("symbol engine is busy; stack unavailable\n");
    }
    InterlockedExchange(&reporting, 0);
    return EXCEPTION_EXECUTE_HANDLER;
}

__declspec(dllexport) DWORD platform_io_windows_crash_prepare(HANDLE process) {
    AcquireSRWLockExclusive(&lifecycle_lock);
    if (initialization == 2) {
        DWORD error = process == requested_process ? ERROR_SUCCESS : ERROR_INVALID_PARAMETER;
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return error;
    }
    HANDLE session = NULL;
    HANDLE caller = GetCurrentProcess();
    if (!DuplicateHandle(caller, process, caller, &session, 0, FALSE, DUPLICATE_SAME_ACCESS)) {
        DWORD error = GetLastError();
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : error;
    }
    AcquireSRWLockExclusive(&symbol_lock);
    DWORD error = ERROR_SUCCESS;
    if (!SymInitialize(session, NULL, TRUE)) {
        error = GetLastError();
        if (error == ERROR_SUCCESS) error = ERROR_GEN_FAILURE;
        if (!CloseHandle(session))
            WRITE_TEXT("symbol session close failed after initialization error\n");
    } else {
        requested_process = process;
        symbol_process = session;
        previous_error_mode = SetErrorMode(GetErrorMode() | SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
        initialization = 2;
    }
    ReleaseSRWLockExclusive(&symbol_lock);
    ReleaseSRWLockExclusive(&lifecycle_lock);
    return error;
}

__declspec(dllexport) void platform_io_windows_crash_install(LPTOP_LEVEL_EXCEPTION_FILTER handler) {
    AcquireSRWLockExclusive(&lifecycle_lock);
    LPTOP_LEVEL_EXCEPTION_FILTER current = SetUnhandledExceptionFilter(handler);
    if (current != installed_filter) previous_filter = current;
    installed_filter = handler;
    ReleaseSRWLockExclusive(&lifecycle_lock);
}

__declspec(dllexport) DWORD platform_io_windows_crash_finish(void) {
    AcquireSRWLockExclusive(&lifecycle_lock);
    if (initialization != 2) {
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return ERROR_INVALID_STATE;
    }
    AcquireSRWLockExclusive(&symbol_lock);
    if (installed_filter != NULL) {
        LPTOP_LEVEL_EXCEPTION_FILTER current = SetUnhandledExceptionFilter(previous_filter);
        if (current != installed_filter) SetUnhandledExceptionFilter(current);
    }
    SetErrorMode(previous_error_mode);
    DWORD error = ERROR_SUCCESS;
    if (!SymCleanup(symbol_process)) {
        error = GetLastError();
        if (error == ERROR_SUCCESS) error = ERROR_GEN_FAILURE;
    }
    if (!CloseHandle(symbol_process)) {
        DWORD close_error = GetLastError();
        if (error == ERROR_SUCCESS)
            error = close_error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : close_error;
        else
            WRITE_TEXT("symbol session close failed after symbol cleanup error\n");
    }
    initialization = 0;
    requested_process = NULL;
    symbol_process = NULL;
    installed_filter = NULL;
    previous_filter = NULL;
    ReleaseSRWLockExclusive(&symbol_lock);
    ReleaseSRWLockExclusive(&lifecycle_lock);
    return error;
}

__declspec(dllexport) int platform_io_windows_crash_section(const void *pointer, int *readonly, const char **name, uint64_t *length) {
    *readonly = 0;
    *name = NULL;
    *length = 0;
    const unsigned char *base = (const unsigned char *)GetModuleHandleW(NULL);
    if (base == NULL || pointer == NULL) return 0;
    const IMAGE_DOS_HEADER *dos = (const IMAGE_DOS_HEADER *)base;
    if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew <= 0) return 0;
    const IMAGE_NT_HEADERS64 *nt = (const IMAGE_NT_HEADERS64 *)(base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE || nt->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC) return 0;
    uintptr_t address = (uintptr_t)pointer;
    if (address < (uintptr_t)base || address - (uintptr_t)base >= nt->OptionalHeader.SizeOfImage) return 0;
    uintptr_t offset = address - (uintptr_t)base;
    const IMAGE_SECTION_HEADER *section = IMAGE_FIRST_SECTION(nt);
    for (unsigned i = 0; i < nt->FileHeader.NumberOfSections; ++i) {
        uint64_t start = section[i].VirtualAddress;
        uint64_t size = section[i].Misc.VirtualSize;
        if (start + size > nt->OptionalHeader.SizeOfImage) return 0;
        if (offset >= start && offset - start < size) {
            *readonly = (section[i].Characteristics & IMAGE_SCN_MEM_WRITE) == 0;
            *name = (const char *)section[i].Name;
            while (*length < IMAGE_SIZEOF_SHORT_NAME && section[i].Name[*length] != 0) ++*length;
            return 1;
        }
    }
    return 0;
}
