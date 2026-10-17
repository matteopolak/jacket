#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";
S :: #import "String";

main :: () {
    subbands: [32][3] s32;
    output: [80] float;

    for *output it.* = 37;
    for *subbands it.*[1] = 1;

    M.audio_matrix_transform(subbands, 1, output.data, 8);
    assert(output[7] == 37 && output[72] == 37);
    assert(output[8 + 48] < -31.999 && output[8 + 48] > -32.001);
    assert(output[8 + 16] > -0.00001 && output[8 + 16] < 0.00001);
    for *subbands it.*[1] = 0;

    subbands[0][1] = 1024;
    M.audio_matrix_transform(subbands, 1, output.data, 0);
    assert(output[0] > 724.07 && output[0] < 724.09 && output[48] == -1024);

    allocation_bits := u8.[0x10];
    probe: M.Audio;
    probe.buffer = M.buffer_create_with_memory(allocation_bits, false);
    quantizer := M.audio_read_allocation(*probe, 0, 1);
    table := *M.QUANT_TAB;

    assert(quantizer == table.data && quantizer.levels == 3 && quantizer.bits == 5);
    M.buffer_destroy(probe.buffer);

    // MPEG-1 Layer II, 128 kbit/s, 44.1 kHz, mono, unprotected. Zero bit
    // allocation in every subband reconstructs genuine silence.
    bytes: [417] u8;
    bytes[0] = 0xff;
    bytes[1] = 0xfd;
    bytes[2] = 0x80;
    bytes[3] = 0xc0;
    buffer := M.buffer_create_with_memory(bytes, false);
    audio := M.audio_create_with_buffer(buffer, true);

    assert(audio.has_header && audio.samplerate_index == 0 && audio.mode == .MONO);
    samples := M.audio_decode(audio);

    assert(samples != null && samples.count == 1152 && samples.time == 0);
    for samples.interleaved assert(it == 0);

    assert(audio.samples_decoded == 1152);
    assert(M.audio_decode(audio) == null);
    M.audio_rewind(audio);
    samples = M.audio_decode(audio);
    assert(samples != null && samples.time == 0);
    for samples.interleaved assert(it == 0);

    M.audio_destroy(audio);
    filename := sprint("%/fixtures/clip.mpg", S.path_strip_filename(#file));

    defer free(filename);
    clip, loaded := F.read_entire_file(filename);

    assert(loaded && clip.count == 6144);
    defer free(clip);
    input := cast(*u8) clip.data;

    // Explicit extraction of this fixture's two audio PES payloads only.
    assert(input[2054] == 0x21 && input[4114] == 0x21);
    data := cast(*u8) alloc(3343);

    memcpy(data, input + 2059, 2037);
    memcpy(data + 2037, input + 4119, 1306);
    buffer = M.buffer_create_with_memory((.{ data = data, count = 3343 }), true);
    audio = M.audio_create_with_buffer(buffer, true);
    assert(audio.has_header && audio.mode == .MONO && M.SAMPLE_RATE[audio.samplerate_index] == 44100);

    for round: 0..1 {
        blocks: s64;
        total: s64;
        crossings: s64;
        measured: s64;
        previous: float;
        peak: float;

        while true {
            samples = M.audio_decode(audio);
            if !samples break;

            assert(samples.count == 1152 && blocks < 16);

            for i: 0..1151 {
                value := samples.interleaved[i * 2];

                assert(value == samples.interleaved[i * 2 + 1]);
                assert(value >= -1 && value <= 1);
                magnitude := value;
                if magnitude < 0 magnitude = -magnitude;
                if magnitude > peak peak = magnitude;

                // Measure the steady interior: the encoded final frame includes
                // padding/ringing with additional low-amplitude zero crossings.
                if total >= 1024 && total < 16128 {
                    if previous <= 0 && value > 0 crossings += 1;
                    measured += 1;
                }

                previous = value;
                total += 1;
            }

            blocks += 1;
        }

        assert(blocks == 16 && total == 18432 && audio.samples_decoded == total);
        assert(peak > 0.1 && peak < 0.2);
        assert(measured == 15104);
        frequency := cast(float64) crossings * 44100 / measured;

        assert(frequency > 430 && frequency < 450);
        if round == 0 M.audio_rewind(audio);
    }

    M.audio_destroy(audio);
    print("ok\n");
}
