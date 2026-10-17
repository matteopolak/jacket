#import "Basic";

// An expanded `__reg` parameter binds any register declared by #asm, not only a vector one.
add_regs :: (c: __reg, d: __reg) #expand {
    #asm {
        add c, d;
    }
}

main :: () {
    #asm {
        mov a:, 10;
        mov b:, 7;
    }

    add_regs(b, a);

    c: u64 = ---;
    #asm {
        mov c, b;
    }

    assert(c == 17, "c: %", c);
    print("ok\n");
}
