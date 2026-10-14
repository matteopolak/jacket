M :: #import "pl_mpeg";

main :: () {
    video: M.Video;
    video.mb_width = 1;
    video.mb_height = 1;
    source: [64] u8;
    dest: [64] u8;

    M.video_process_macroblock(*video, dest.data, source.data, -1, 0, 8, false);
}
