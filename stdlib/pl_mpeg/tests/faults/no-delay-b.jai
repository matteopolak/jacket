// test: should_fail("MPEG B picture violates no-delay assumption")
#import "Basic";
M :: #import "pl_mpeg";
#import "Extensions/Environment";

main :: () {
    filename := output_path("portable-tests/mpeg/reordered.m1v");
    buffer := M.buffer_create_with_filename(filename);

    assert(buffer != null);
    video := M.video_create_with_buffer(buffer, true);

    M.video_set_no_delay(video, true);
    while M.video_decode(video) {}
}
