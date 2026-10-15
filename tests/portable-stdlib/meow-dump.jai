#import "Basic";
M :: #import "meow_hash";

main :: () {
    seed: [128] u8;
    for 0..15 seed[it * 8] = cast(u8) (it + 1);

    #asm {
        pxor h0:, h0; pxor h1:, h1; pxor h2:, h2; pxor h3:, h3;
        pxor h4:, h4; pxor h5:, h5; pxor h6:, h6; pxor h7:, h7;
    }

    M.MeowBegin(seed, h0, h1, h2, h3, h4, h5, h6, h7);
    M.MEOW_DUMP_STATE("state", h0, h1, h2, h3, h4, h5, h6, h7);
    print("ok\n");
}
