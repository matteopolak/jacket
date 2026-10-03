// Independent ARM64 implementation using official native Windows APIs.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <stdint.h>
#include <string.h>

#if !defined(_M_ARM64) && !defined(__aarch64__)
#error This private backend requires native Windows ARM64
#endif

// Microsoft documents dynamic resolution and this ABI; it has no required
// user-mode import library/header declaration. Windows 11/Server 2022 minimum.
typedef LONG (NTAPI *unwind_function)(DWORD, DWORD64, DWORD64, PRUNTIME_FUNCTION,
    PCONTEXT, PBOOLEAN, PVOID *, PDWORD64, PKNONVOLATILE_CONTEXT_POINTERS,
    PDWORD64, PDWORD64, PEXCEPTION_ROUTINE *, DWORD);

static SRWLOCK lifecycle_lock = SRWLOCK_INIT;
static SRWLOCK symbol_lock = SRWLOCK_INIT;
static DWORD references;
static LONG reporting;
static HANDLE symbol_process;
static unwind_function unwind_frame;
static LPTOP_LEVEL_EXCEPTION_FILTER previous_filter;
static UINT previous_error_mode;

static void write_bytes(const char *data, DWORD count) {
    HANDLE output = GetStdHandle(STD_ERROR_HANDLE);
    while (count != 0) {
        DWORD written = 0;
        if (!WriteFile(output, data, count, &written, NULL) || written == 0)
            ExitProcess(ERROR_WRITE_FAULT);
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

static void write_frame(DWORD64 address) {
    WRITE_TEXT("  ");
    write_hex(address);
    _Alignas(SYMBOL_INFO) unsigned char storage[sizeof(SYMBOL_INFO) + 1024] = {0};
    SYMBOL_INFO *symbol = (SYMBOL_INFO *)storage;
    symbol->SizeOfStruct = sizeof(SYMBOL_INFO);
    symbol->MaxNameLen = 1024;
    DWORD64 displacement = 0;
    if (SymFromAddr(symbol_process, address, &displacement, symbol)) {
        WRITE_TEXT(" ");
        DWORD length = symbol->NameLen;
        if (length > 1023) length = 1023;
        write_bytes(symbol->Name, length);
        WRITE_TEXT(" + ");
        write_hex(displacement);
    }
    WRITE_TEXT("\n");
}

static void walk_stack(const CONTEXT *source) {
    // SDK context only. Never copy this value into the public x64 record.
    CONTEXT context;
    memcpy(&context, source, sizeof(context));
    ULONG_PTR low, high;
    GetCurrentThreadStackLimits(&low, &high);
    DWORD64 low_limit = low, high_limit = high;
    for (unsigned depth = 0; depth < 64 && context.Pc != 0; ++depth) {
        write_frame(context.Pc);
        if (context.Sp < low || context.Sp >= high || (context.Sp & 15) != 0) {
            WRITE_TEXT("invalid native stack bounds\n");
            return;
        }
        DWORD64 previous_pc = context.Pc, previous_sp = context.Sp;
        DWORD64 image_base = 0, establisher = 0;
        PRUNTIME_FUNCTION function = RtlLookupFunctionEntry(context.Pc, &image_base, NULL);
        PVOID handler_data = NULL;
        LONG status = unwind_frame(UNW_FLAG_NHANDLER, image_base, context.Pc,
            function, &context, NULL, &handler_data, &establisher, NULL,
            &low_limit, &high_limit, NULL, 0);
        if (status != 0) {
            WRITE_TEXT("native unwind failed: ");
            write_hex((uint32_t)status);
            WRITE_TEXT("\n");
            return;
        }
        if (context.Pc == 0) return;
        if (context.Sp < previous_sp || (context.Pc == previous_pc && context.Sp == previous_sp)) {
            WRITE_TEXT("native unwind made no progress\n");
            return;
        }
    }
}

static LONG WINAPI handle_exception(EXCEPTION_POINTERS *exception) {
    if (exception == NULL || exception->ExceptionRecord == NULL || exception->ContextRecord == NULL)
        return EXCEPTION_CONTINUE_SEARCH;
    if (InterlockedCompareExchange(&reporting, 1, 0) != 0) return EXCEPTION_CONTINUE_SEARCH;
    WRITE_TEXT("Windows exception ");
    write_hex(exception->ExceptionRecord->ExceptionCode);
    WRITE_TEXT(" at ");
    write_hex((uintptr_t)exception->ExceptionRecord->ExceptionAddress);
    WRITE_TEXT("\nnative stack:\n");
    // A fault may have interrupted DbgHelp on this or another thread.
    if (TryAcquireSRWLockExclusive(&symbol_lock)) {
        if (symbol_process != NULL) walk_stack(exception->ContextRecord);
        else WRITE_TEXT("symbol engine is not initialized\n");
        ReleaseSRWLockExclusive(&symbol_lock);
    } else {
        WRITE_TEXT("symbol engine is busy; stack unavailable\n");
    }
    InterlockedExchange(&reporting, 0);
    return EXCEPTION_EXECUTE_HANDLER;
}

__declspec(dllexport) DWORD platform_io_arm64_crash_init(void) {
    AcquireSRWLockExclusive(&lifecycle_lock);
    DWORD error = ERROR_SUCCESS;
    if (references != 0) {
        if (references == UINT32_MAX) error = ERROR_ARITHMETIC_OVERFLOW;
        else ++references;
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return error;
    }
    HMODULE ntdll = GetModuleHandleW(L"ntdll.dll");
    FARPROC entry = ntdll == NULL ? NULL : GetProcAddress(ntdll, "RtlVirtualUnwind2");
    if (entry == NULL) {
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return ERROR_PROC_NOT_FOUND;
    }
    // Copy a native function pointer representation, avoiding incompatible casts.
    _Static_assert(sizeof(entry) == sizeof(unwind_frame), "native function pointer ABI");
    memcpy(&unwind_frame, &entry, sizeof(entry));
    HANDLE process = GetCurrentProcess();
    HANDLE session = NULL;
    if (!DuplicateHandle(process, process, process, &session, 0, FALSE, DUPLICATE_SAME_ACCESS)) {
        error = GetLastError();
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : error;
    }
    AcquireSRWLockExclusive(&symbol_lock);
    if (!SymInitialize(session, NULL, TRUE)) {
        error = GetLastError();
        if (error == ERROR_SUCCESS) error = ERROR_GEN_FAILURE;
        if (!CloseHandle(session)) {
            // Preserve the primary error, and report the independent cleanup failure.
            WRITE_TEXT("failed to close symbol session after initialization error\n");
        }
    } else {
        symbol_process = session;
        previous_error_mode = SetErrorMode(GetErrorMode() | SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
        previous_filter = SetUnhandledExceptionFilter(handle_exception);
        references = 1;
    }
    ReleaseSRWLockExclusive(&symbol_lock);
    ReleaseSRWLockExclusive(&lifecycle_lock);
    return error;
}

__declspec(dllexport) DWORD platform_io_arm64_crash_fini(void) {
    AcquireSRWLockExclusive(&lifecycle_lock);
    if (references == 0) {
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return ERROR_INVALID_STATE;
    }
    if (--references != 0) {
        ReleaseSRWLockExclusive(&lifecycle_lock);
        return ERROR_SUCCESS;
    }
    AcquireSRWLockExclusive(&symbol_lock);
    LPTOP_LEVEL_EXCEPTION_FILTER current = SetUnhandledExceptionFilter(previous_filter);
    // Preserve a filter installed by the application after ours. The native API
    // has no compare/exchange operation; external replacement must be serialized.
    if (current != handle_exception) SetUnhandledExceptionFilter(current);
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
            WRITE_TEXT("failed to close symbol session after symbol cleanup error\n");
    }
    symbol_process = NULL;
    ReleaseSRWLockExclusive(&symbol_lock);
    ReleaseSRWLockExclusive(&lifecycle_lock);
    return error;
}
