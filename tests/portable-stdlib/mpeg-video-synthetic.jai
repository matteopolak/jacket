#import "Basic";
M :: #import "pl_mpeg";
S :: #import "String";

main :: () {
    root := sprint("%/../../target/portable-tests/mpeg/", S.path_strip_filename(#file));

    defer free(root);
    filename := sprint("%reordered.m1v", root);

    defer free(filename);
    buffer := M.buffer_create_with_filename(filename);

    assert(buffer != null);
    video := M.video_create_with_buffer(buffer, true);
    expected := u8.[80, 120, 160];

    for round: 0..1 {
        for index: 0..2 {
            frame := M.video_decode(video);
            assert(frame != null && frame.time == cast(float64) index / 25);
            for 0..255 assert(frame.y.data[it] == expected[index]);
            for 0..63 assert(frame.cb.data[it] == 128 && frame.cr.data[it] == 128);
        }

        assert(M.video_decode(video) == null);
        if round == 0 M.video_rewind(video);
    }

    M.video_destroy(video);
    filename_ac := sprint("%ac.m1v", root);

    defer free(filename_ac);
    buffer = M.buffer_create_with_filename(filename_ac);
    assert(buffer != null);
    video = M.video_create_with_buffer(buffer, true);
    frame := M.video_decode(video);

    assert(frame != null);
    // One horizontal AC term: identical rows, decreasing luma across the first
    // block, symmetric around its DC level; other blocks remain flat.
    assert(frame.y.data[0] > 130 && frame.y.data[7] < 126);

    for row: 0..7 for col : 0..7 {
        value := cast(s32) frame.y.data[row * 16 + col];
        mirror := cast(s32) frame.y.data[row * 16 + 7 - col];

        assert(value == frame.y.data[col]);
        assert(value + mirror >= 255 && value + mirror <= 257);
        if col < 7 assert(value >= frame.y.data[row * 16 + col + 1]);
    }

    for row: 0..15 for col : 0..15 {
        if row >= 8 || col >= 8 assert(frame.y.data[row * 16 + col] == 128);
    }

    assert(M.video_decode(video) == null);
    M.video_destroy(video);
    print("ok\n");
}
