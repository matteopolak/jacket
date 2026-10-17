#import "Basic";
M :: #import "pl_mpeg";
S :: #import "String";

main :: () {
    filename := sprint("%/../../../../target/portable-tests/mpeg/reordered.m1v", S.path_strip_filename(#file));

    defer free(filename);
    buffer := M.buffer_create_with_filename(filename);

    assert(buffer != null);
    video := M.video_create_with_buffer(buffer, true);

    M.video_set_no_delay(video, true);
    while M.video_decode(video) {}
}
