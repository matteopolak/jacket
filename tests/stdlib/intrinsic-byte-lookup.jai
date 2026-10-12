// `byte_lookup` and `sign_mask`: byte-table lookup and sign-bit mask over 16 and 32 bytes.
// byte_lookup gives table[i] for an index i below 16 and 0 for every larger index.
#import "Basic";

byte_lookup :: (dest: *u8, table: *u8, indices: *u8, width: s64) #intrinsic;
sign_mask :: (source: *u8, width: s64) -> u64 #intrinsic;

fail :: (what: string, a: int, b: int) {
    print("FAIL %: % %\n", what, a, b);
    exit(1);
}

check_lookup16 :: (table: *[16] u8, indices: *[16] u8) {
    out: [16] u8;

    byte_lookup(out.data, table.data, indices.data, 16);

    for 0..15 {
        want: u8 = 0;
        if indices.*[it] < 16 want = table.*[indices.*[it]];
        if out[it] != want fail("lookup16", it, out[it]);
    }
}

check_lookup32 :: (table: *[16] u8, indices: *[32] u8) {
    out: [32] u8;

    byte_lookup(out.data, table.data, indices.data, 32);

    for 0..31 {
        want: u8 = 0;
        if indices.*[it] < 16 want = table.*[indices.*[it]];
        if out[it] != want fail("lookup32", it, out[it]);
    }
}

check_lookup64 :: (table: *[16] u8, indices: *[64] u8) {
    out: [64] u8;

    byte_lookup(out.data, table.data, indices.data, 64);

    for 0..63 {
        want: u8 = 0;
        if indices.*[it] < 16 want = table.*[indices.*[it]];
        if out[it] != want fail("lookup64", it, out[it]);
    }
}

main :: () {
    table: [16] u8;
    for 0..15 table[it] = xx (it * 7 + 3);

    // Every index value 0..255, in 16-byte batches.
    for base: 0..15 {
        idx: [16] u8;
        for 0..15 idx[it] = xx (base * 16 + it);
        check_lookup16(*table, *idx);
    }

    // 32 bytes: both blocks use the same table.
    for base: 0..7 {
        idx: [32] u8;
        for 0..31 idx[it] = xx (base * 32 + it);
        check_lookup32(*table, *idx);
    }

    // 64 bytes: four blocks, one table.
    for base: 0..3 {
        idx: [64] u8;
        for 0..63 idx[it] = xx (base * 64 + it);
        check_lookup64(*table, *idx);
    }

    // Reversed indices.
    rev: [32] u8;
    for 0..31 rev[it] = xx (15 - (it % 16));

    check_lookup32(*table, *rev);

    // Sign mask.
    v16: [16] u8;
    for 0..15 v16[it] = ifx it % 3 == 0 then 0x80 else xx it;

    want16: u64;
    for 0..15 if v16[it] & 0x80 want16 |= (cast(u64) 1) << it;

    got := sign_mask(v16.data, 16);
    if got != want16 fail("mask16", xx got, xx want16);

    v32: [32] u8;
    for 0..31 v32[it] = ifx it % 5 == 0 || it == 31 then 0xFF else xx it;

    want32: u64;
    for 0..31 if v32[it] & 0x80 want32 |= (cast(u64) 1) << it;

    got = sign_mask(v32.data, 32);
    if got != want32 fail("mask32", xx got, xx want32);

    v64: [64] u8;
    for 0..63 v64[it] = ifx it % 7 == 0 || it == 63 then 0x81 else xx it;

    want64: u64;
    for 0..63 if v64[it] & 0x80 want64 |= (cast(u64) 1) << it;

    got = sign_mask(v64.data, 64);
    if got != want64 fail("mask64", xx got, xx want64);

    zeros: [16] u8;
    if sign_mask(zeros.data, 16) != 0 fail("mask zero", 0, 0);

    ones: [16] u8;
    for 0..15 ones[it] = 0x80;
    if sign_mask(ones.data, 16) != 0xFFFF fail("mask ones", 0, 0);

    print("ok\n");
}
