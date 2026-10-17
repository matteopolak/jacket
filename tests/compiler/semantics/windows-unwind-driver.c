// Original SDK runtime regression. Run on native Windows; cross-linking is not execution.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <assert.h>
#include <stdint.h>
#include <stdio.h>

typedef int (*capture_fn)(void);
extern int64_t unwind_middle(int64_t);
extern int unwind_entry(void);
extern int unwind_capture_leaf(capture_fn);
extern int unwind_capture_middle(capture_fn);

static uintptr_t function_begin(uintptr_t address) {
    DWORD64 base = 0;
    PRUNTIME_FUNCTION function = RtlLookupFunctionEntry((DWORD64)address, &base, NULL);
    assert(function != NULL);
    return (uintptr_t)(base + function->BeginAddress);
}

static int capture(void) {
    void *frames[64];
    USHORT count = CaptureStackBackTrace(0, 64, frames, NULL);
    uintptr_t leaf = function_begin((uintptr_t)unwind_capture_leaf);
    uintptr_t middle = function_begin((uintptr_t)unwind_capture_middle);
    int leaf_index = -1, middle_index = -1;
    for (USHORT i = 0; i < count; ++i) {
        DWORD64 base = 0;
        PRUNTIME_FUNCTION function = RtlLookupFunctionEntry((DWORD64)(uintptr_t)frames[i], &base, NULL);
        if (function == NULL) continue; // Native true leaf frames legitimately have no table.
        uintptr_t begin = (uintptr_t)(base + function->BeginAddress);
        if (begin == leaf) leaf_index = i;
        if (begin == middle) middle_index = i;
    }
    assert(leaf_index >= 0 && middle_index > leaf_index);
    return 17;
}

int main(void) {
    assert(unwind_entry() == 0);
    assert(unwind_middle(8) == 41);
    (void)function_begin((uintptr_t)unwind_middle);
    assert(unwind_capture_middle(capture) == 19);
    puts("real Windows frames and unwind tables: ok");
    return 0;
}
