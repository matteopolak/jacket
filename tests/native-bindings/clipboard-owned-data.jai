// Independent allocator ownership regression for the reviewed public fields.
// No X server or fabricated native Xlib calls are used in this portable test.
#import "Basic";
Owned :: #import "internal/bindings/clipboard-owned-data";

Buffer :: struct {
    text_data: string;
    rgb_data: *u8;
    width: s32;
    height: s32;
    pitch: s32;
}

Tracking :: struct {
    previous: Allocator;
    text: *void;
    pixels: *void;
    text_frees: s64;
    pixel_frees: s64;
}

track :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;

    if mode == .FREE && old_memory {
        if old_memory == state.text state.text_frees += 1;
        else if old_memory == state.pixels state.pixel_frees += 1;
        else assert(false, "unexpected owned-buffer free");
    }

    return state.previous.proc(mode, requested_size, old_size, old_memory, state.previous.data);
}

main :: () {
    guarded: struct {
        before: u64 = 0x716;
        buffer: Buffer;
        after: u64 = 0x918;
    };

    guarded.buffer.text_data = alloc_string(17);
    guarded.buffer.rgb_data = cast(*u8) alloc(24);
    guarded.buffer.width = 3;
    guarded.buffer.height = 2;
    guarded.buffer.pitch = 12;
    state: Tracking;
    state.previous = context.allocator;
    state.text = guarded.buffer.text_data.data;
    state.pixels = guarded.buffer.rgb_data;
    context.allocator = .{ track, *state };
    defer context.allocator = state.previous;
    Owned.release(*guarded.buffer);
    assert(state.text_frees == 1 && state.pixel_frees == 1);
    assert(guarded.buffer.text_data.data == null && guarded.buffer.text_data.count == 0);
    assert(guarded.buffer.rgb_data == null);
    assert(guarded.buffer.width == 0 && guarded.buffer.height == 0 && guarded.buffer.pitch == 0);
    assert(guarded.before == 0x716 && guarded.after == 0x918);
    Owned.release(*guarded.buffer);
    assert(state.text_frees == 1 && state.pixel_frees == 1);
}
