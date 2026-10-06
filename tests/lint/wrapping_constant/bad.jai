#import "Basic";

main :: () {
    h: s32 = 7;
    a := (0xffff_ffff - 40) / h;
    below := h < 0x8000_0000;
    u: u16 = 9;
    r := u % -2;
    h %= 0x8000_0000;
    print("% % % %\n", a, below, r, h);
}
