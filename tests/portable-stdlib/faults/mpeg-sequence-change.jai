M :: #import "pl_mpeg";
#import "Basic";

main :: () {
    bytes := u8.[0, 0, 1, 0xb3, 0x01, 0, 0x10, 0x13, 0, 0, 0x20, 0,
                  0, 0, 1, 0xb3, 0x02, 0, 0x10, 0x13, 0, 0, 0x20, 0];
    buffer := M.buffer_create_with_memory(bytes, false);
    video := M.video_create_with_buffer(buffer, true);

    assert(video.has_sequence_header);
    M.video_decode(video);
}
