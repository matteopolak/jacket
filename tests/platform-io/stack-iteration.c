// Pure iterator unit tests. Scripted transitions are NOT native unwind evidence.
#include "../../stdlib/internal/runtime/crash-native/stack-iteration.h"
#include <assert.h>
#include <stddef.h>

typedef struct {
    const platform_stack_frame *frames;
    size_t count, at, emitted;
    platform_stack_frame output[64];
} script;

static int next(void *data, platform_stack_frame *frame) {
    script *state = data;
    if (state->at == state->count) return 0;
    *frame = state->frames[state->at++];
    return 1;
}

static void emit(void *data, platform_stack_frame frame) {
    script *state = data;
    assert(state->emitted < 64);
    state->output[state->emitted++] = frame;
}

int main(void) {
    platform_stack_frame first = {1, 100};
    const platform_stack_frame duplicate_first[] = {{1, 100}, {2, 120}, {3, 140}};
    script state = {duplicate_first, 3, 0, 0, {{0, 0}}};
    assert(platform_stack_iterate(first, 0, &state, next, emit) == PLATFORM_STACK_END);
    assert(state.emitted == 3 && state.output[0].pc == 1 && state.output[1].pc == 2 && state.output[2].pc == 3);
    state = (script){duplicate_first, 3, 0, 0, {{0, 0}}};
    assert(platform_stack_iterate(first, 1, &state, next, emit) == PLATFORM_STACK_END);
    assert(state.emitted == 2 && state.output[0].pc == 2 && state.output[1].pc == 3);
    const platform_stack_frame recursion[] = {{1, 120}, {1, 140}};
    state = (script){recursion, 2, 0, 0, {{0, 0}}};
    assert(platform_stack_iterate(first, 0, &state, next, emit) == PLATFORM_STACK_END);
    assert(state.emitted == 3);
    const platform_stack_frame cycle[] = {{2, 120}, {1, 100}};
    state = (script){cycle, 2, 0, 0, {{0, 0}}};
    assert(platform_stack_iterate(first, 0, &state, next, emit) == PLATFORM_STACK_STALLED);
    assert(state.emitted == 2);
    platform_stack_frame long_trace[64];
    for (unsigned i = 0; i < 64; ++i) long_trace[i] = (platform_stack_frame){i + 2, 120 + i * 20};
    state = (script){long_trace, 64, 0, 0, {{0, 0}}};
    assert(platform_stack_iterate(first, 0, &state, next, emit) == PLATFORM_STACK_LIMIT);
    assert(state.emitted == 64 && state.at == 63 && state.output[63].pc == 64);
    return 0;
}
