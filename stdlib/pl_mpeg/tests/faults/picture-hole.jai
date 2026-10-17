M :: #import "pl_mpeg";

main :: () {
    // A 32x16 intra picture encodes only the second macroblock. Reaching the
    // final address must not make the absent first macroblock a decoded frame.
    stream := u8.[0, 0, 1, 0xb3, 0x02, 0, 0x10, 0x13, 0, 0, 0x20, 0,
                  0, 0, 1, 0, 0, 0x0f, 0xff, 0xf8,
                  0, 0, 1, 1, 0x41, 0xe5, 0x29, 0x48, 0x88];
    buffer := M.buffer_create_with_memory(stream, false);
    video := M.video_create_with_buffer(buffer, true);

    M.video_decode(video);
}
