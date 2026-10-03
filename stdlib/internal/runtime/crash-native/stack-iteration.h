// Private SDK-independent traversal control. Native SDK calls supply every frame.
#ifndef PLATFORM_IO_STACK_ITERATION_H
#define PLATFORM_IO_STACK_ITERATION_H
#include <stdint.h>

typedef struct {
    uint64_t pc;
    uint64_t sp;
} platform_stack_frame;

enum platform_stack_result { PLATFORM_STACK_END, PLATFORM_STACK_STALLED, PLATFORM_STACK_LIMIT };
typedef int (*platform_stack_advance)(void *, platform_stack_frame *);
typedef void (*platform_stack_emit)(void *, platform_stack_frame);

static enum platform_stack_result platform_stack_iterate(platform_stack_frame frame,
    int skips, void *data, platform_stack_advance advance, platform_stack_emit emit) {
    if (frame.pc == 0) return PLATFORM_STACK_END;
    platform_stack_frame seen[64];
    seen[0] = frame;
    unsigned count = 1;
    if (skips <= 0) emit(data, frame);
    for (unsigned step = 0; count < 64; ++step) {
        if (!advance(data, &frame) || frame.pc == 0) return PLATFORM_STACK_END;
        // The initial SDK call may establish its internal state while returning
        // the starting frame. That is not a cycle, nor an additional frame.
        if (step == 0 && frame.pc == seen[0].pc && frame.sp == seen[0].sp) continue;
        for (unsigned i = 0; i < count; ++i) {
            if (frame.pc == seen[i].pc && frame.sp == seen[i].sp)
                return PLATFORM_STACK_STALLED;
        }
        seen[count] = frame;
        if (skips <= 0 || count >= (unsigned)skips) emit(data, frame);
        ++count;
    }
    return PLATFORM_STACK_LIMIT;
}
#endif
