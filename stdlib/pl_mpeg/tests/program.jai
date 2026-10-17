#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";
S :: #import "String";

Counts :: struct {
    video, audio: s64;
}

Tracking :: struct {
    base: Allocator;
    live: s64;
}

tracked :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);

    if mode == .ALLOCATE && result state.live += 1;
    if mode == .RESIZE && !old_memory && result state.live += 1;
    if mode == .FREE && old_memory state.live -= 1;

    return result;
}

on_video :: (self: *M.Plm, frame: *M.Frame, user: *void) {
    counts := cast(*Counts) user;
    assert(self != null && frame.width == 32 && frame.height == 32);
    counts.video += 1;
}

on_audio :: (plm: *M.Plm, samples: *M.Samples, user: *void) {
    counts := cast(*Counts) user;
    assert(plm != null && samples.count == 1152);
    counts.audio += 1;
}

main :: () {
    state := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ tracked, *state };
    push_context scoped run_cases();
    assert(state.live == 0);
    print("ok\n");
}

run_cases :: () {
    #assert !#exists(M.Demux);
    filename := sprint("%/fixtures/clip.mpg", S.path_strip_filename(#file));

    defer free(filename);
    plm := M.create_with_filename(filename);

    assert(plm != null && M.has_headers(plm));
    assert((cast(*Type_Info_Struct) type_info(M.Plm)).name == "Plm");
    width, height := M.get_size(plm);

    assert(width == 32 && height == 32 && M.get_framerate(plm) == 25 && M.get_samplerate(plm) == 44100);
    assert(M.get_num_video_streams(plm) == 1);
    count: s64;
    rgb: [3072] u8;

    while true {
        frame := M.decode_video(plm);
        if !frame break;

        M.frame_to_rgb(frame, rgb.data, 96);
        for p: 0..1023 assert(rgb[p * 3] > 235 && rgb[p * 3 + 1] < 20 && rgb[p * 3 + 2] < 20);

        count += 1;
        assert(count <= 10);
    }

    assert(count == 10 && plm.has_ended);
    count = 0;

    while M.decode_audio(plm) {
        count += 1;
        assert(count <= 16);
    }

    assert(count == 16);
    M.rewind(plm);
    M.set_loop(plm, true);
    count = 0;

    while M.decode_video(plm) {
        count += 1;
        assert(count <= 10);
    }

    assert(count == 10 && !plm.has_ended && plm.time == 0);
    assert(M.decode_video(plm) != null);
    M.set_loop(plm, false);
    M.rewind(plm);
    counts: Counts;

    M.set_video_decode_callback(plm, on_video, *counts);
    M.set_audio_decode_callback(plm, on_audio, *counts);
    M.decode(plm, 0.2);
    assert(counts.video == 5 && counts.audio == 8 && !plm.has_ended);
    M.decode(plm, 1.0);
    assert(counts.video == 10 && counts.audio == 16 && plm.has_ended);
    M.rewind(plm);
    M.set_audio_enabled(plm, false);
    assert(M.decode_audio(plm) == null);
    frame := M.seek_frame(plm, 0.13, true);

    assert(frame != null && frame.time == 0.16 && !plm.has_ended);
    frame = M.seek_frame(plm, 0.13, false);
    assert(frame != null && frame.time == 0.12);
    M.plm_destroy(plm);

    fh, opened := F.file_open(filename);

    assert(opened);
    plm = M.create_with_file(fh, false);
    assert(plm != null && M.has_headers(plm));
    M.plm_destroy(plm);
    assert(F.file_set_position(fh, 0));
    prefix: [4] u8;
    read_ok, read_count := F.file_read(fh, prefix.data, prefix.count);

    assert(read_ok && read_count == 4 && prefix[3] == 0xba);
    F.file_close(*fh);

    clip, loaded := F.read_entire_file(filename);

    assert(loaded);
    defer free(clip);
    plm = M.create_with_memory((.{ data = cast(*u8) clip.data, count = clip.count }), false);
    assert(plm != null && M.has_headers(plm));
    M.set_video_enabled(plm, false);
    assert(M.decode_video(plm) == null && M.decode_audio(plm) != null);
    M.plm_destroy(plm);
    source := M.create_for_appending(1);

    plm = M.create_with_buffer(source, false);
    assert(plm != null && !M.has_headers(plm));

    for 0..clip.count - 1 {
        assert(M.buffer_write(source, (.{ data = cast(*u8) clip.data + it, count = 1 })) == 1);
        M.has_headers(plm);
    }

    M.buffer_signal_end(source);
    assert(M.has_headers(plm));
    count = 0;

    while M.decode_video(plm) {
        count += 1;
        assert(count <= 10);
    }

    assert(count == 10);
    M.plm_destroy(plm);
    M.buffer_rewind(source);
    assert(M.buffer_read(source, 32) == 0x000001ba);
    M.buffer_destroy(source);

    // More than the default source-buffer capacity, exercising refills while
    // packet parsing retains its pending start code and compacts consumed bytes.
    repeated := cast(*u8) alloc(clip.count * 24);

    defer free(repeated);
    for 0..23 memcpy(repeated + it * clip.count, clip.data, clip.count);

    large_filename := sprint("%/../../../target/portable-tests/mpeg-refill.mpg", S.path_strip_filename(#file));

    defer free(large_filename);
    assert(F.write_entire_file(large_filename, (.{ data = cast(*u8) repeated, count = clip.count * 24 })));
    plm = M.create_with_filename(large_filename);
    assert(plm != null && M.has_headers(plm));
    M.set_audio_enabled(plm, false);
    count = 0;

    while M.decode_video(plm) {
        count += 1;
        assert(count <= 240);
    }

    assert(count == 240);
    M.plm_destroy(plm);
    assert(F.file_delete(large_filename));
    assert(M.create_with_filename("no-such-portable-mpeg-clip") == null);
    junk: [32] u8;

    plm = M.create_with_memory(junk, false);
    assert(plm != null && !M.has_headers(plm));
    M.plm_destroy(plm);
}
