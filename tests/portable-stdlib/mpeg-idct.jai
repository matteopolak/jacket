#import "Basic";
M :: #import "pl_mpeg";

main :: () {
    block: [64] s32;

    M.video_idct(block.data);
    for block assert(it == 0);

    block[0] = 256 * 37;
    M.video_idct(block.data);
    for block assert(it == 37);
    for *block it.* = 0;

    block[0] = -256 * 19;
    M.video_idct(block.data);
    for block assert(it == -19);
    for *block it.* = 0;

    block[1] = 256;
    M.video_idct(block.data);
    expected := s32.[1, 1, 1, 0, 0, -1, -1, -1];
    for row: 0..7 for col : 0..7 assert(block[row * 8 + col] == expected[col]);

    print("ok\n");
}
