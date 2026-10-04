#import "Basic";

// #asm lowers x64 instructions to plain operations on Jai variables, so these
// run unchanged in the interpreter, in the browser and in native builds.

test_mov_arith :: () {
    x: s64 = 10;
    #asm {
        add x, 5;
        sub x, 3;
    }
    assert(x == 12, "add/sub");

    #asm {
        t: gpr;
        mov.q t, 100;
        add x, t;
        mov u:, 7;
        sub x, u;
    }
    assert(x == 105, "gpr temporaries persist in the enclosing scope");

    // Writes narrower than the variable.
    y: u64 = 0x1122334455667788;
    #asm { mov.d y, 5; }
    assert(y == 5, "32-bit writes zero-extend");
    z: u64 = 0x1122334455667788;
    #asm { mov.w z, 5; }
    assert(z == 0x1122334455660005, "16-bit writes merge");
    w: u64 = 0x1122334455667788;
    #asm { mov.b w, 0xff; }
    assert(w == 0x11223344556677ff, "8-bit writes merge");

    // Immediates of every size.
    #asm {
        mov.b b1:, 255;
        mov.b b2:, -127;
        mov.w w1:, 65535;
        mov.d d2:, -2147483648;
        mov.q q3:, 18446744073709551615;
        mov.d f1:, 1.5;
        mov.q f2:, 2.5;
    }
    big: u64 = ---;
    #asm { mov big, q3; }
    assert(big == 0xffffffffffffffff, "imm64");
    f32_bits: u32 = ---;
    #asm { mov f32_bits, f1; }
    assert(f32_bits == 0x3fc00000, "float32 immediate");
    f64_bits: u64 = ---;
    #asm { mov f64_bits, f2; }
    assert(f64_bits == 0x4004000000000000, "float64 immediate");
}

test_carry :: () {
    lo: u64 = 0xffffffffffffffff;
    hi: u64 = 0;
    #asm {
        add lo, 1;
        adc hi, 0;
    }
    assert(lo == 0 && hi == 1, "add/adc 128-bit");

    #asm {
        sub lo, 1;
        sbb hi, 0;
    }
    assert(lo == 0xffffffffffffffff && hi == 0, "sub/sbb 128-bit");

    small: u8 = 250;
    carry: bool = ---;
    #asm {
        add.b small, 10;
        setc carry;
    }
    assert(small == 4 && carry, "8-bit carry out");

    a: s32 = 0x7fffffff;
    overflow: bool = ---;
    #asm {
        add.d a, 1;
        seto overflow;
    }
    assert(a == -2147483648 && overflow, "signed overflow");
}

test_logic :: () {
    v: u64 = 0xf0;
    #asm {
        and.q v, 15;
        or.q v, 32;
        shl.q v, 1;
        shr.q v;
    }
    assert(v == 32, "and/or/shl/shr by 1");

    a: u32 = 0xff00ff00;
    #asm { xor a, 0xffffffff; }
    assert(a == 0x00ff00ff, "xor");
    #asm { not a; }
    assert(a == 0xff00ff00, "not");

    n: s64 = 5;
    #asm { neg n; }
    assert(n == -5, "neg");
    #asm {
        inc n;
        inc n;
        dec n;
    }
    assert(n == -4, "inc/dec");
}

test_shifts :: () {
    s: s32 = -64;
    #asm { sar.d s, 2; }
    assert(s == -16, "sar");

    u: u32 = 0x80000001;
    #asm { rol.d u, 1; }
    assert(u == 3, "rol");
    #asm { ror.d u, 1; }
    assert(u == 0x80000001, "ror");

    count: u8 = 4;
    k: u64 = 1;
    #asm { shl.q k, count; }
    assert(k == 16, "shift by a variable count");

    byte: u8 = 0x81;
    #asm { shl.b byte, 1; }
    assert(byte == 2, "8-bit shift");
    out: bool = ---;
    byte = 0x81;
    #asm {
        shl.b byte, 1;
        setc out;
    }
    assert(out, "shift carry out");

    zero: u64 = 3;
    z: u8 = 0;
    flag: bool = true;
    #asm {
        xor flag_tmp:, flag_tmp;  // CF = 0
        shl.q zero, z;            // count 0 keeps the flags
        setc flag;
    }
    assert(zero == 3 && !flag, "zero count");
}

test_compare :: () {
    a: s64 = -1;
    b: s64 = 1;
    lt: bool = ---;
    below: bool = ---;
    eq: bool = ---;
    ge: bool = ---;
    #asm {
        cmp a, b;
        setl lt;
        setb below;
        sete eq;
        setge ge;
    }
    assert(lt && !below && !eq && !ge, "cmp signed/unsigned conditions");

    c: u32 = 7;
    zero: bool = ---;
    nzero: bool = ---;
    #asm {
        test.d c, 8;
        setz zero;
        test.d c, 4;
        setnz nzero;
    }
    assert(zero && nzero, "test");

    max: s64 = 5;
    min: s64 = 9;
    #asm {
        cmp max, min;
        cmovl max, min;  // max = max < min ? min : max
    }
    assert(max == 9, "cmov");

    gt: bool = ---;
    ab: bool = ---;
    x: u64 = 10;
    #asm {
        cmp x, 3;
        setg gt;
        seta ab;
    }
    assert(gt && ab, "greater");
}

test_multiply :: () {
    a: s64 = -7;
    #asm { imul a, 6; }
    assert(a == -42, "imul two operands");
    r: s64 = ---;
    #asm { imul r, a, 5; }
    assert(r == -210, "imul with immediate");

    x: u64 = 197589578578;
    y: u64 = 895173299817;
    z: u64 = ---;
    #asm {
        x === a;
        z === d;
        mul z, x, y;
    }
    assert(z == 9588 && x == 9532886391493626018, "mul 64x64 -> 128");

    p: u32 = 0xffffffff;
    q: u32 = 0xffffffff;
    hi: u32 = ---;
    #asm { mul.d hi, p, q; }
    assert(hi == 0xfffffffe && p == 1, "mul 32x32 -> 64");

    m: s64 = -3;
    n: s64 = 4;
    shi: s64 = ---;
    #asm { imul shi, m, n; }
    assert(shi == -1 && m == -12, "signed 64x64 -> 128");
}

test_bits :: () {
    out: u32 = ---;
    v: u32 = 0xf0f0;
    #asm { popcnt.d out, v; }
    assert(out == 8, "popcnt");

    one: u32 = 1;
    #asm { lzcnt.d out, one; }
    assert(out == 31, "lzcnt");
    zero: u32 = 0;
    #asm { lzcnt.d out, zero; }
    assert(out == 32, "lzcnt of zero");

    eight: u64 = 8;
    r64: u64 = ---;
    #asm { tzcnt.q r64, eight; }
    assert(r64 == 3, "tzcnt");

    bits: u32 = 0x50;
    #asm { bsf.d out, bits; }
    assert(out == 4, "bsf");
    #asm { bsr.d out, bits; }
    assert(out == 6, "bsr");

    // bsf keeps the destination for a zero source (bit_scan_forward relies on it).
    out = 77;
    found: bool = ---;
    #asm {
        bsf.d out, zero;
        setz found;
    }
    assert(out == 77 && found, "bsf of zero");

    sw: u32 = 0x11223344;
    #asm { bswap.d sw; }
    assert(sw == 0x44332211, "bswap.d");
    sw64: u64 = 0x1122334455667788;
    #asm { bswap.q sw64; }
    assert(sw64 == 0x8877665544332211, "bswap.q");
    sw16: u16 = 0xff00;
    #asm { bswap.w sw16; }
    assert(sw16 == 0x00ff, "bswap.w");

    a: u64 = 0b1100;
    d: u64 = ---;
    #asm { blsr.q d, a; }
    assert(d == 0b1000, "blsr");
    #asm { blsi.q d, a; }
    assert(d == 0b0100, "blsi");
    #asm { blsmsk.q d, a; }
    assert(d == 0b0111, "blsmsk");
}

test_bit_test :: () {
    x: u32 = 0b1010;
    c: bool = ---;
    #asm {
        bt x, 1;
        setc c;
    }
    assert(c && x == 0b1010, "bt");
    #asm {
        bts x, 0;
        setc c;
    }
    assert(!c && x == 0b1011, "bts");
    #asm {
        btr x, 1;
        setc c;
    }
    assert(c && x == 0b1001, "btr");
    #asm { btc x, 3; }
    assert(x == 0b0001, "btc");

    // Register bit offsets on memory address a bit string.
    words: [2] u32;
    p := words.data;
    idx: s64 = 33;
    #asm {
        bts [p], idx;
        setc c;
    }
    assert(!c && words[0] == 0 && words[1] == 2, "bts bit string");

    // The File_Async pattern.
    available: u32 = 0b0110;
    ptr := *available;
    bit_index: u32 = 2;
    success: bool = ---;
    #asm {
        lock_btr.d [ptr], bit_index;
        setc success;
    }
    assert(success && available == 0b0010, "lock_btr");
    #asm {
        lock_btr.d [ptr], bit_index;
        setc success;
    }
    assert(!success, "lock_btr of a clear bit");
    #asm { lock_bts.d [ptr], bit_index; }
    assert(available == 0b0110, "lock_bts");
}

test_lea_memory :: () {
    a: s64 = 100;
    b: s64 = 3;
    r: s64 = ---;
    #asm {
        lea.q r, [a + b*8 + 16];
    }
    assert(r == 140, "lea");
    #asm { lea.q r, [a - 4]; }
    assert(r == 96, "lea negative displacement");
    rax := 5;
    rdx := 7;
    #asm { lea.q rax, [rdx + rax*4]; }
    assert(rax == 27, "lea with register-named variables");

    array: [8] u32;
    for 0..7 array[it] = cast(u32) (it * 10);
    p := array.data;
    i: s64 = 3;
    v: u32 = ---;
    #asm {
        mov.d v, [p + i*4];
    }
    assert(v == 30, "load [base + index*scale]");
    #asm { mov.d [p + 8], 99; }
    assert(array[2] == 99, "store immediate");
    #asm {
        mov t:, 1234;
        mov.d [p + i*4 + 4], t;
    }
    assert(array[4] == 1234, "store register");

    SCALE :: 4;
    DISP :: -4;
    #asm { mov.d v, [p + i*SCALE + DISP + 8]; }
    assert(v == array[3 + 1], "constant scale and displacement");
    #asm { mov.d v, [*array + 12]; }
    assert(v == 30, "address-of base");

    bytes: [4] u8 = .[0x80, 0, 0, 0];
    bp := bytes.data;
    wide: s32 = ---;
    narrow: u64 = ---;
    #asm {
        movzxbd narrow, [bp];
        movsxbd wide, [bp];
    }
    assert(narrow == 0x80 && wide == -128, "movzx/movsx");
}

test_atomic_forms :: () {
    counter: s64 = 100;
    p := *counter;
    #asm {
        mov incr: gpr, 5;
        lock_xadd.q [p], incr;
    }
    assert(counter == 105, "lock_xadd");
    old: s64 = ---;
    #asm {
        mov old, incr;
    }
    assert(old == 100, "xadd returns the old value");

    #asm { lock_inc.q [p]; }
    #asm { lock_inc.q [p]; }
    #asm { lock_dec.q [p]; }
    assert(counter == 106, "lock_inc/lock_dec");
    zero: bool = ---;
    counter = 1;
    #asm {
        lock_dec.q [p];
        setz zero;
    }
    assert(zero, "lock_dec sets ZF");

    mask: u32 = 0xff;
    value: u32 = 0x0f;
    vp := *value;
    #asm { lock_or.d [vp], mask; }
    assert(value == 0xff, "lock_or");
    mask = 0x3c;
    #asm { lock_and.d [vp], mask; }
    assert(value == 0x3c, "lock_and");
    #asm { lock_xor.d [vp], mask; }
    assert(value == 0, "lock_xor");

    // xchg
    a: s32 = 1;
    b: s32 = 2;
    #asm { xchg a, b; }
    assert(a == 2 && b == 1, "xchg registers");
    cell: s32 = 9;
    cp := *cell;
    v: s32 = 4;
    #asm { lock_xchg.d v, [cp]; }
    assert(cell == 4 && v == 9, "xchg memory");

    // cmpxchg: (destination, new value, accumulator)
    target: s64 = 50;
    tp := *target;
    expected: s64 = 50;
    new_value: s64 = 60;
    ok: bool = ---;
    #asm {
        lock_cmpxchg.q [tp], new_value, expected;
        setz ok;
    }
    assert(ok && target == 60, "cmpxchg success");
    expected = 50;
    #asm {
        lock_cmpxchg.q [tp], new_value, expected;
        setz ok;
    }
    assert(!ok && target == 60 && expected == 60, "cmpxchg failure loads the accumulator");
}

add_regs :: (c: __reg, d: __reg) #expand {
    #asm {
        add c, d;
    }
}

test_macros :: () {
    #asm {
        mov a:, 10;
        mov b:, 7;
    }
    add_regs(b, a);
    c: u64 = ---;
    #asm { mov c, b; }
    assert(c == 17, "__reg macro parameters");

    fetch_and_add :: (val: *int) #expand {
        #asm {
            mov incr: gpr, 1;
            xadd.q [val], incr;
        }
    }
    g := 108;
    fetch_and_add(*g);
    assert(g == 109, "fetch_and_add macro");
}

popbit :: (a: u64) -> u64 #expand {
    #asm { blsr.q a, a; }
    return a;
}

bit_count :: (value: $T) -> s32 {
    result: s32;
    #asm { popcnt?T result, value; }
    return result;
}

do_some_work :: (a: int, b: int) -> int {
    #asm { add a, b; }
    return a;
}

COMPILE_TIME :: #run do_some_work(10, 13);

test_misc :: () {
    assert(popbit(0xff) == 0xfe, "popbit macro");
    assert(bit_count(cast(u32) 0xff00ff) == 16, "?T size from a type");
    assert(bit_count(cast(u64) 0xffffffffffff) == 48, "?T size 64");
    assert(COMPILE_TIME == 23, "#asm at compile time");

    #asm {
        nop;
        pause;
        mfence;
    }
    for 1..3 #asm { pause; }

    #asm { rdtsc hi:, lo:; }
    first: u32 = ---;
    #asm { mov first, lo; }
    #asm { rdtsc hi2:, lo2:; }
    assert(hi == hi2 || hi2 > hi, "rdtsc is monotonic");

    #asm {
        mov a:, 1;
        mov c:, 0;
        cpuid a, b:, c, d:;
    }
    r: u32 = ---;
    #asm { mov r, b; }
    assert(r == 0, "cpuid reports no features");

    flagged: bool = ---;
    #asm {
        stc;
        setc flagged;
    }
    assert(flagged, "stc");
    #asm {
        clc;
        setc flagged;
    }
    assert(!flagged, "clc");
}

main :: () {
    test_mov_arith();
    test_carry();
    test_logic();
    test_shifts();
    test_compare();
    test_multiply();
    test_bits();
    test_bit_test();
    test_lea_memory();
    test_atomic_forms();
    test_macros();
    test_misc();
    print("asm ok\n");
}
