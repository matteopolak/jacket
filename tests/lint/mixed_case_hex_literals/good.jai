#import "Basic";

MASK :: 0xdeadbeef;

// A macro may paste literals it was given.
shifted :: () -> u32 #expand {
    return 0xAbCd;
}

main :: () {
    a := 0xFFFF;
    b: u32 = 0xabcdef01;
    c := 0x1234;
    d := 0xFF_EE_DD;
    e := 1234;
    f := 0b1010;

    print("% % % % % % %\n", MASK, a, b, c, d, e, f);
    print("%\n", shifted());
}
