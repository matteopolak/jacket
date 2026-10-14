#import "Basic";
M :: #import "pl_mpeg";
S :: #import "String";

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

main :: () {
    state := Tracking.{ base = context.allocator };
    scoped := context;
    scoped.allocator = .{ tracked, *state };
    push_context scoped run_cases();
    assert(state.live == 0);
    print("ok\n");
}

run_cases :: () {
    #assert !#exists(M.read_quantized);
    #assert !#exists(M.scale_samples);
    #assert !#exists(M.Audio_Private);
    #assert !#exists(M.crc_bits);
    root := sprint("%/../../target/portable-tests/mpeg/", S.path_strip_filename(#file));

    defer free(root);
    names := string.["stereo", "dual", "joint", "joint-crc"];
    for names {
        filename := sprint("%%.mp2", root, it);
        buffer := M.buffer_create_with_filename(filename);

        free(filename);
        assert(buffer != null);
        audio := M.audio_create_with_buffer(buffer, true);

        assert(audio.has_header && audio.mode != .MONO);

        for round: 0..1 {
            peak: float;

            for block: 0..1 {
                samples := M.audio_decode(audio);

                assert(samples != null && samples.count == 1152);

                for i: 0..1151 {
                    left := samples.interleaved[i * 2];
                    right := samples.interleaved[i * 2 + 1];

                    assert(left >= -1 && left <= 1 && right >= -1 && right <= 1);
                    difference := right - left * 0.25;

                    assert(difference > -0.000001 && difference < 0.000001);
                    magnitude := left;
                    if magnitude < 0 magnitude = -magnitude;
                    if magnitude > peak peak = magnitude;
                }
            }

            assert(peak > 0.01);
            assert(M.audio_decode(audio) == null);
            if round == 0 M.audio_rewind(audio);
        }

        M.audio_destroy(audio);
    }

}
