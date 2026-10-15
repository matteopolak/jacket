#import "Basic";
M :: #import "pl_mpeg";
S :: #import "String";

reports: s64;

Tracking :: struct {
    base: Allocator;
    outstanding: s64;
    fail_size: s64;
    failures: s64;
}

tracking_allocator :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    if mode == .ALLOCATE && requested_size == state.fail_size {
        state.failures += 1;
        return null;
    }

    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);
    if mode == .ALLOCATE && result state.outstanding += 1;
    if mode == .RESIZE && !old_memory && result state.outstanding += 1;
    if mode == .FREE && old_memory state.outstanding -= 1;

    return result;
}

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message == "invalid MPEG sequence header\n" && (info.common_flags & .ERROR) != 0);
    reports += 1;
}

main :: () {
    tracking := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ tracking_allocator, *tracking };
    push_context scoped run_cases(*tracking);
    assert(tracking.outstanding == 0);
    print("ok\n");
}

run_cases :: (tracking: *Tracking) {
    bytes := u8.[0, 0, 1, 0xb3, 0x01, 0x00, 0x10, 0x13, 0, 0, 0x20, 0];
    buffer := M.buffer_create_with_memory(bytes, false);
    video := M.video_create_with_buffer(buffer, false);

    assert(video != null && video.has_sequence_header);
    assert((cast(*Type_Info_Struct) type_info(M.Video)).name == "Video");
    assert(video.width == 16 && video.height == 16 && M.video_get_framerate(video) == 25);
    assert(video.mb_width == 1 && video.mb_height == 1 && video.mb_size == 1);
    assert(video.frame_current.width == 16 && video.frame_current.y.width == 16);
    assert(video.frame_current.cr.data == video.frames_data + 256);
    assert(video.frame_current.cb.data == video.frames_data + 320);
    assert(video.frame_forward.y.data == video.frames_data + 384);
    assert(video.frame_backward.y.data == video.frames_data + 768);
    assert(video.intra_quant_matrix[0] == 8 && video.intra_quant_matrix[63] == 83);
    assert(video.non_intra_quant_matrix[5] == 16);
    M.video_set_time(video, 1.5);
    assert(video.time == 1.5 && video.frames_decoded == 37);
    M.video_set_no_delay(video, true);
    assert(video.assume_no_b_frames);
    M.video_rewind(video);
    assert(video.time == 0 && video.frames_decoded == 0 && !video.has_reference_frame);
    assert(M.video_has_header(video));
    M.video_destroy(video);
    M.buffer_destroy(buffer);

    tracking.fail_size = 1152;
    buffer = M.buffer_create_with_memory(bytes, false);
    video = M.video_create_with_buffer(buffer, true);
    assert(!video.has_sequence_header && !video.frames_data && buffer.bit_index == 32 && tracking.failures == 1);
    tracking.fail_size = 0;
    assert(M.video_has_header(video) && video.frames_data != null);
    M.video_destroy(video);

    buffer = M.create_for_appending(12);
    assert(M.buffer_write(buffer, (.{ data = bytes.data, count = 9 })) == 9);
    video = M.video_create_with_buffer(buffer, true);
    assert(!video.has_sequence_header && buffer.bit_index == 32);
    assert(M.buffer_write(buffer, (.{ data = bytes.data + 9, count = 3 })) == 3);
    assert(M.video_has_header(video));
    assert(buffer.bit_index == 96 && video.width == 16);
    M.video_destroy(video);
    custom: [140] u8;
    custom[2] = 1;
    custom[3] = 0xb3;
    bit: s64 = 32;

    put(custom, *bit, 17, 12);
    put(custom, *bit, 31, 12);
    put(custom, *bit, 1, 4);
    put(custom, *bit, 5, 4);
    put(custom, *bit, 0, 18);
    put(custom, *bit, 1, 1);
    put(custom, *bit, 0, 11);
    put(custom, *bit, 1, 1);
    for 0..63 put(custom, *bit, cast(u32) (it + 1), 8);

    put(custom, *bit, 1, 1);
    for 0..63 put(custom, *bit, cast(u32) (64 - it), 8);

    assert(bit == custom.count * 8);
    buffer = M.create_for_appending(1);
    video = M.video_create_with_buffer(buffer, true);

    for 0..139 {
        assert(M.buffer_write(buffer, (.{ data = custom.data + it, count = 1 })) == 1);
        ready := M.video_has_header(video);
        assert(ready == (it == 139));
        if !ready assert(!video.frames_data && video.width == 0);
    }

    assert(video.width == 17 && video.height == 31 && video.mb_size == 4);
    assert(video.framerate == 30 && video.frame_current.y.width == 32);

    for 0..63 {
        assert(video.intra_quant_matrix[M.ZIG_ZAG[it]] == it + 1);
        assert(video.non_intra_quant_matrix[M.ZIG_ZAG[it]] == 64 - it);
    }

    replacement := u8.[0, 0, 1, 0xb3, 0x01, 0x10, 0x1f, 0x15, 0, 0, 0x20, 0];
    storage := video.frames_data;

    video.has_reference_frame = true;
    assert(M.buffer_write(buffer, replacement) == replacement.count);
    assert(M.buffer_find_start_code(buffer, .SEQUENCE));
    assert(M.video_decode_sequence_header(video));
    assert(video.frames_data == storage && video.has_reference_frame);

    for 0..63 {
        assert(video.intra_quant_matrix[it] == M.INTRA_QUANT_MATRIX[it]);
        assert(video.non_intra_quant_matrix[it] == M.NON_INTRA_QUANT_MATRIX[it]);
    }

    M.video_destroy(video);
    bytes[10] = 0;
    previous := context.logger;

    context.logger = capture;
    buffer = M.buffer_create_with_memory(bytes, false);
    video = M.video_create_with_buffer(buffer, true);
    assert(!video.has_sequence_header && !video.frames_data && buffer.bit_index == 32 && reports == 1);
    context.logger = previous;
    M.video_destroy(video);
    filename := sprint("%/../native-regressions/pl-mpeg-clip.mpg", S.path_strip_filename(#file));

    defer free(filename);
    buffer = M.buffer_create_with_filename(filename);
    assert(buffer != null);
    video = M.video_create_with_buffer(buffer, true);
    assert(video.has_sequence_header && video.width == 32 && video.height == 32 && video.framerate == 25);
    M.video_destroy(video);
}

put :: (bytes: [] u8, index: *s64, value: u32, count: s64) {
    for 0..count - 1 {
        bit := (value >> (count - it - 1)) & 1;
        bytes[index.* >> 3] |= cast(u8) (bit << (7 - (index.* & 7)));
        index.* += 1;
    }
}
