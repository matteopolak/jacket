// Original native SDK traversal probe: reports the first StackWalk64 transition.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>

static HANDLE session;

static unsigned named_leaf_address(DWORD64 address) {
    _Alignas(SYMBOL_INFO) unsigned char storage[sizeof(SYMBOL_INFO) + 256] = {0};
    SYMBOL_INFO *symbol = (SYMBOL_INFO *)storage;
    symbol->SizeOfStruct = sizeof(SYMBOL_INFO);
    symbol->MaxNameLen = 256;
    DWORD64 displacement = 0;
    return SymFromAddr(session, address, &displacement, symbol) &&
           strstr(symbol->Name, "sdk_walk_leaf") != NULL;
}

__declspec(dllexport) __declspec(noinline) unsigned sdk_walk_leaf(void) {
    CONTEXT context;
    RtlCaptureContext(&context);
    STACKFRAME64 frame = {0};
    frame.AddrPC.Offset = context.Rip;
    frame.AddrStack.Offset = context.Rsp;
    frame.AddrFrame.Offset = context.Rbp;
    frame.AddrPC.Mode = frame.AddrStack.Mode = frame.AddrFrame.Mode = AddrModeFlat;
    DWORD64 last_pc = frame.AddrPC.Offset, last_sp = frame.AddrStack.Offset;
    unsigned distinct = 0, steps = 0, first_same = 0;
    unsigned named_leaf = named_leaf_address(context.Rip);
    while (steps < 64 && StackWalk64(IMAGE_FILE_MACHINE_AMD64, session,
           GetCurrentThread(), &frame, &context, NULL, SymFunctionTableAccess64,
           SymGetModuleBase64, NULL)) {
        if (frame.AddrPC.Offset == 0) break;
        if (steps == 0)
            first_same = frame.AddrPC.Offset == last_pc && frame.AddrStack.Offset == last_sp;
        if (frame.AddrPC.Offset != last_pc || frame.AddrStack.Offset != last_sp) ++distinct;
        last_pc = frame.AddrPC.Offset;
        last_sp = frame.AddrStack.Offset;
        named_leaf |= named_leaf_address(frame.AddrPC.Offset);
        ++steps;
    }
    printf("SDK StackWalk64: first_same=%u steps=%u distinct=%u named_leaf=%u\n",
           first_same, steps, distinct, named_leaf);
    assert(distinct >= 3 && named_leaf);
    return distinct;
}

__declspec(dllexport) __declspec(noinline) unsigned sdk_walk_middle(void) {
    unsigned count = sdk_walk_leaf();
    return count + 1; // Preserve the caller; this is deliberately not a tail call.
}

int main(void) {
    HANDLE process = GetCurrentProcess();
    assert(DuplicateHandle(process, process, process, &session, 0, FALSE, DUPLICATE_SAME_ACCESS));
    assert(SymInitialize(session, NULL, TRUE));
    assert(sdk_walk_middle() >= 4);
    assert(SymCleanup(session));
    assert(CloseHandle(session));
    return 0;
}
