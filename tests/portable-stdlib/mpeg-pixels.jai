#import "Basic";
M :: #import "pl_mpeg";

main :: () {
    assert(M.SYNTHESIS_WINDOW.count == 512 && M.DCT_COEFF.count == 224);
    assert(M.ZIG_ZAG.count == 64 && M.QUANT_TAB.count == 17);
    assert(M.SYNTHESIS_WINDOW[0] == 0 && M.SYNTHESIS_WINDOW[1] == -0.5);
    assert(M.QUANT_TAB[16].levels == 65535);
    assert(M.IS_SLICE(.SLICE_FIRST) && M.IS_SLICE(.SLICE_LAST));
    assert(!M.IS_SLICE(.PICTURE) && !M.IS_SLICE(.SEQUENCE));
    assert(M.clamp(-1) == 0 && M.clamp(42) == 42 && M.clamp(256) == 255);
    y := u8.[16, 235, 77, 99, 81, 145, 88, 99];
    cr := u8.[128, 99];
    cb := u8.[128, 99];
    frame: M.Frame;
    frame.width = 2;
    frame.height = 2;
    frame.y = .{ 4, 2, y.data };
    frame.cr = .{ 2, 1, cr.data };
    frame.cb = .{ 2, 1, cb.data };
    dest: [20] u8;
    for *dest it.* = 37;

    M.frame_to_rgba(*frame, dest.data, 10);
    assert(dest[0] == 0 && dest[1] == 0 && dest[2] == 0 && dest[3] == 37);
    assert(dest[4] == 254 && dest[5] == 254 && dest[6] == 254 && dest[7] == 37);
    assert(dest[8] == 37 && dest[9] == 37 && dest[18] == 37 && dest[19] == 37);
    assert(dest[10] == 75 && dest[14] == 150);
    cr[0] = 240;
    cb[0] = 90;
    y[0] = 81;
    rgb: [12] u8;
    bgr: [12] u8;

    M.frame_to_rgb(*frame, rgb.data, 6);
    M.frame_to_bgr(*frame, bgr.data, 6);
    assert(rgb[0] == 253 && rgb[1] == 0 && rgb[2] == 0);

    for 0..3 {
        assert(rgb[it * 3] == bgr[it * 3 + 2]);
        assert(rgb[it * 3 + 1] == bgr[it * 3 + 1]);
        assert(rgb[it * 3 + 2] == bgr[it * 3]);
    }

    M.frame_to_argb(*frame, dest.data, 10);
    assert(dest[0] == 0 && dest[1] == 253 && dest[2] == 0 && dest[3] == 0);
    M.frame_to_abgr(*frame, dest.data, 10);
    assert(dest[1] == 0 && dest[2] == 0 && dest[3] == 253);
    M.frame_to_bgra(*frame, dest.data, 10);
    assert(dest[0] == 0 && dest[1] == 0 && dest[2] == 253);
    print("ok\n");
}
