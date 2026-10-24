#import "Basic";

MASK :: 0xDeadbeef;

main :: () {
    a := 0xFFfF;
    b: u32 = 0xaBCDEF01;
    c := 0xFF_ee_dd;

    print("% % % %\n", MASK, a, b, c);
}
