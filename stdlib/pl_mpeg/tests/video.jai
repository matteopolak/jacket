#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";
S :: #import "String";

// One valid 16x16 intra picture with zero differential DC and no AC values.
// Six blocks reconstruct their initial predictors (128); no decoder oracle is
// needed for this elementary bitstream's constant luma/chroma result.
main :: () {
    #assert !#exists(M.decode_slice_inner);
    #assert !#exists(M.read_bits);
    stream := u8.[0, 0, 1, 0xb3, 0x01, 0, 0x10, 0x13, 0, 0, 0x20, 0,
                  0, 0, 1, 0, 0, 0x0f, 0xff, 0xf8,
                  0, 0, 1, 1, 0x43, 0x94, 0xa5, 0x22, 0x20];
    buffer := M.buffer_create_with_memory(stream, false);
    video := M.video_create_with_buffer(buffer, true);

    assert(video.has_sequence_header);
    frame := M.video_decode(video);

    assert(frame != null && frame.width == 16 && frame.height == 16);
    for 0..255 assert(frame.y.data[it] == 128);
    for 0..63 assert(frame.cb.data[it] == 128 && frame.cr.data[it] == 128);

    assert(frame.time == 0 && video.frames_decoded == 1 && video.time == 0.04);
    assert(M.video_decode(video) == null);
    M.video_rewind(video);
    frame = M.video_decode(video);
    assert(frame != null && frame.y.data[0] == 128 && frame.time == 0);
    M.video_destroy(video);
    buffer = M.buffer_create_with_capacity(stream.count);
    assert(M.buffer_write(buffer, stream) == stream.count);
    M.buffer_signal_end(buffer);
    video = M.video_create_with_buffer(buffer, true);
    frame = M.video_decode(video);
    assert(frame != null && frame.y.data[0] == 128);
    assert(M.video_decode(video) == null);
    M.video_destroy(video);
    filename := sprint("%/fixtures/clip.mpg", S.path_strip_filename(#file));

    defer free(filename);
    bytes, loaded := F.read_entire_file(filename);

    assert(loaded && bytes.count == 6144);
    defer free(bytes);
    // This synthetic fixture has one video PES at 30 with a 351-byte packet and
    // ten bytes of PTS/DTS metadata. Restrict the elementary decoder to its video
    // payload; this fixture extraction is not program-stream demux coverage.
    data := cast(*u8) bytes.data;

    assert(data[30] == 0 && data[31] == 0 && data[32] == 1 && data[33] == 0xe0);
    assert(data[34] == 1 && data[35] == 95 && (data[36] >> 4) == 3);
    buffer = M.buffer_create_with_memory((.{ data = data + 46, count = 341 }), false);
    video = M.video_create_with_buffer(buffer, true);
    rgb: [32 * 32 * 3] u8;

    for round: 0..1 {
        count: s64;

        while true {
            frame = M.video_decode(video);
            if !frame break;

            assert(frame.width == 32 && frame.height == 32);
            M.frame_to_rgb(frame, rgb.data, 32 * 3);

            for pixel: 0..1023 {
                assert(rgb[pixel * 3] > 235 && rgb[pixel * 3 + 1] < 20 && rgb[pixel * 3 + 2] < 20);
            }

            count += 1;
            assert(count <= 10);
        }

        assert(count == 10 && video.frames_decoded == 10);

        if round == 0 {
            M.video_rewind(video);
            M.video_set_no_delay(video, true);
        }
    }

    M.video_destroy(video);
    print("ok\n");
}
