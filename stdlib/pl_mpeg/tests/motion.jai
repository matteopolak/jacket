#import "Basic";
M :: #import "pl_mpeg";

main :: () {
    video: M.Video;
    video.mb_width = 2;
    video.mb_height = 2;
    source: [256] u8;
    dest: [256] u8;

    for *source it.* = cast(u8) it_index;

    M.video_process_macroblock(*video, dest.data, source.data, 1, 1, 8, false);
    for row: 0..7 for col : 0..7 assert(dest[row * 16 + col] == row * 16 + col + 9);

    M.video_process_macroblock(*video, dest.data, source.data, 1, 0, 8, false);
    assert(dest[0] == 1);
    M.video_process_macroblock(*video, dest.data, source.data, 0, 1, 8, false);
    assert(dest[0] == 8);
    for *dest it.* = 100;

    M.video_process_macroblock(*video, dest.data, source.data, 1, 1, 8, true);
    assert(dest[0] == 55 && dest[8] == 100);
    video.mb_row = 1;
    video.mb_col = 1;
    M.video_process_macroblock(*video, dest.data, source.data, -1, -1, 8, false);
    assert(dest[136] == 128 && dest[135] == 100);

    bits := u8.[0x58];

    video.buffer = M.buffer_create_with_memory(bits, false);
    assert(M.video_decode_motion_vector(*video, 2, 63) == -61);
    M.buffer_destroy(video.buffer);
    bits[0] = 0x78;
    video.buffer = M.buffer_create_with_memory(bits, false);
    assert(M.video_decode_motion_vector(*video, 2, -64) == 60);
    M.buffer_destroy(video.buffer);
    bits[0] = 0x80;
    video.buffer = M.buffer_create_with_memory(bits, false);
    assert(M.video_decode_motion_vector(*video, 0, 5) == 5);
    video.picture_type = .PREDICTIVE;
    video.motion_forward.h = 5;
    video.motion_forward.v = -3;
    M.video_decode_motion_vectors(*video);
    assert(video.motion_forward.h == 0 && video.motion_forward.v == 0);
    M.buffer_destroy(video.buffer);
    frames: [4608] u8;

    video.width = 32;
    video.height = 32;
    video.luma_width = 32;
    video.luma_height = 32;
    video.chroma_width = 16;
    video.chroma_height = 16;
    video.mb_row = 0;
    video.mb_col = 0;
    M.video_init_frame(*video, *video.frame_current, frames.data);
    M.video_init_frame(*video, *video.frame_forward, frames.data + 1536);
    M.video_init_frame(*video, *video.frame_backward, frames.data + 3072);

    for 0..1535 {
        frames[1536 + it] = 20;
        frames[3072 + it] = 80;
    }

    video.picture_type = .B;
    video.motion_forward.is_set = 1;
    video.motion_backward.is_set = 1;
    M.video_predict_macroblock(*video);
    assert(video.frame_current.y.data[0] == 50);
    assert(video.frame_current.cr.data[0] == 50 && video.frame_current.cb.data[0] == 50);
    video.motion_forward.is_set = 0;
    M.video_predict_macroblock(*video);
    assert(video.frame_current.y.data[0] == 80 && video.frame_current.cb.data[0] == 80);
    video.picture_type = .PREDICTIVE;
    M.video_predict_macroblock(*video);
    assert(video.frame_current.y.data[0] == 20 && video.frame_current.cr.data[0] == 20);
    for 0..1535 frames[1536 + it] = cast(u8) (it % 256);

    video.motion_forward.full_px = 1;
    video.motion_forward.h = 1;
    M.video_predict_macroblock(*video);
    // A full-pixel vector of one becomes two half-pixel units, hence x=1.
    assert(video.frame_current.y.data[0] == 1 && video.frame_current.cb.data[0] == 1);
    print("ok\n");
}
