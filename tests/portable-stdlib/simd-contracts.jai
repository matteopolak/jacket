#import "Basic";
#import "SIMD";
S :: #import "SIMD";

subtract_wrapping :: (a: $T, b: T) -> T #no_aoc {
    return a - b;
}

exercise :: ($T: Type) {
    a := S.set(4, cast(T) 0);
    b := S.set(4, cast(T) 0);

    a.data = .[4, 6, 8, 10];
    b.data = .[1, 6, 4, 12];
    sum := a + b;
    difference := a - b;
    product := a * b;
    quotient := a / b;
    scalar_sum := a + cast(T) 2;
    reversed_sum := cast(T) 2 + a;
    scalar_difference := a - cast(T) 2;
    scalar_product := a * cast(T) 2;
    reversed_product := cast(T) 2 * a;
    scalar_quotient := a / cast(T) 2;
    equal := a == b;
    unequal := a != b;
    less := a < b;
    less_equal := a <= b;
    greater := a > b;
    greater_equal := a >= b;
    scalar_equal := a == cast(T) 6;
    scalar_unequal := a != cast(T) 6;
    scalar_less := a < cast(T) 6;
    scalar_less_equal := a <= cast(T) 6;
    scalar_greater := a > cast(T) 6;
    scalar_greater_equal := a >= cast(T) 6;
    minimum := S.min(a, b);
    maximum := S.max(a, b);
    scalar_minimum := S.min(a, cast(T) 6);
    scalar_maximum := S.max(a, cast(T) 6);

    for index: 0..3 {
        av := a.data[index];
        bv := b.data[index];

        if sum.data[index] != av + bv || product.data[index] != av * bv || quotient.data[index] != av / bv exit(2);

        // Unsigned subtraction wraps under the public #no_aoc contract.
        expected_difference := subtract_wrapping(av, bv);
        if difference.data[index] != expected_difference exit(3);
        if scalar_sum.data[index] != av + 2 || reversed_sum.data[index] != av + 2 exit(4);
        if scalar_difference.data[index] != av - 2 || scalar_product.data[index] != av * 2 || reversed_product.data[index] != av * 2 || scalar_quotient.data[index] != av / 2 exit(5);
        if equal.data[index] != cast(T) (av == bv) || unequal.data[index] != cast(T) (av != bv) exit(6);
        if less.data[index] != cast(T) (av < bv) || less_equal.data[index] != cast(T) (av <= bv) exit(7);
        if greater.data[index] != cast(T) (av > bv) || greater_equal.data[index] != cast(T) (av >= bv) exit(8);
        if scalar_equal.data[index] != cast(T) (av == 6) || scalar_unequal.data[index] != cast(T) (av != 6) exit(9);
        if scalar_less.data[index] != cast(T) (av < 6) || scalar_less_equal.data[index] != cast(T) (av <= 6) exit(10);
        if scalar_greater.data[index] != cast(T) (av > 6) || scalar_greater_equal.data[index] != cast(T) (av >= 6) exit(11);
        if minimum.data[index] != (ifx av < bv then av else bv) || maximum.data[index] != (ifx av > bv then av else bv) exit(12);
        if scalar_minimum.data[index] != (ifx av < 6 then av else cast(T) 6) || scalar_maximum.data[index] != (ifx av > 6 then av else cast(T) 6) exit(13);
    }

    if S.horizontal_sum(a) != 28 || S.get_mask(greater) != 5 || S.get_mask(equal) != 2 exit(14);

    pairs := S.horizontal_add(a, b);
    expected: [4] T = .[10, 7, 18, 16];

    for value, index: expected if pairs.data[index] != value exit(15);
}

main :: () {
    exercise(u8);
    exercise(s8);
    exercise(u16);
    exercise(s16);
    exercise(u32);
    exercise(s32);
    exercise(u64);
    exercise(s64);
    exercise(float32);
    exercise(float64);
    if type_info(S.SIMD_Array(4, u32)).name != "SIMD_Array" || type_info(S.Simd_128).name != "Simd_128" exit(16);

    #assert size_of(S.Simd_128) == 16;
    union_value: S.Simd_128;
    union_value._u32.data[0] = 0x04030201;
    for index: 0..3 if union_value._u8.data[index] != index + 1 exit(17);

    source: [8] u32 = .[11, 22, 33, 44, 55, 66, 77, 88];
    contiguous := S.load(source.data, 4, u32);
    storage: [18] u8;
    storage[0] = 0xa5;
    storage[17] = 0x5a;
    S.store(storage.data + 1, contiguous);
    unaligned := S.load(storage.data + 1, 4, u32);
    for value, index: contiguous.data if unaligned.data[index] != value exit(18);
    if storage[0] != 0xa5 || storage[17] != 0x5a exit(19);

    offsets := S.set(4, cast(s64) 0);
    offsets.data = .[12, 0, 12, 4];
    gathered := S.load_from_offsets(source.data, offsets, u32);
    if gathered.data[0] != 44 || gathered.data[1] != 11 || gathered.data[2] != 44 || gathered.data[3] != 22 exit(20);

    offsets.data = .[-4, -8, 0, 4];
    gathered = S.load_from_offsets(source.data + 4, offsets, u32);
    if gathered.data[0] != 44 || gathered.data[1] != 33 || gathered.data[2] != 55 || gathered.data[3] != 66 exit(21);

    destination: [8] u32;
    for *value: destination value.* = 0xa5a5a5a5;

    offsets.data = .[0, 4, 0, 12];
    S.store_with_offsets(contiguous, destination.data, offsets);
    if destination[0] != 33 || destination[1] != 22 || destination[3] != 44 exit(22);
    for index: s64.[2, 4, 5, 6, 7] if destination[index] != 0xa5a5a5a5 exit(23);

    indices := S.get_looping_indices(4, s32);
    backwards := S.get_looping_indices(4, s32, -3);

    for index: 0..3 if indices.data[index] != index || backwards.data[index] != index * -3 exit(24);

    all := S.set(64, cast(u8) 1);
    if S.get_mask(all) != -1 exit(25);

    write_string("ok\n");
}
