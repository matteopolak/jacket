#import "Basic";

lane :: (value: __reg, $index: int) -> u64 #expand {
    result: u64;
    #asm { pextrq result, value, index; }
    return result;
}

xor_vector :: (target: __reg, value: __reg) #expand {
    #asm { pxor target, value; }
}

main :: () {
    left := u64.[0x0123456789abcdef, 0xfedcba9876543210];
    right := u64.[0x5555555555555555, 0xaaaaaaaaaaaaaaaa];
    lp := left.data;
    rp := right.data;

    #asm {
        movdqu.x a:, [lp];
        movdqu.x b:, [rp];
        pxor cleared:, cleared;
    }

    assert(lane(cleared, 0) == 0 && lane(cleared, 1) == 0);
    assert(lane(a, 0) == left[0] && lane(a, 1) == left[1]);
    xor_vector(a, b);
    assert(lane(a, 0) == (left[0] ^ right[0]));
    assert(lane(a, 1) == (left[1] ^ right[1]));
    print("ok\n");
}
