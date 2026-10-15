#import "Basic";
V :: #import "SIMD";

main :: () {
    a := V.set(4, cast(s32) 3);
    b := V.set(4, cast(s32) 5);
    sum := a + b;
    difference := b - a;
    product := a * b;
    quotient := product / a;

    for 0..3 assert(sum.data[it] == 8 && difference.data[it] == 2 && product.data[it] == 15 && quotient.data[it] == 5);

    scalar := 2 + a * 4;
    for scalar.data assert(it == 14);

    assert(V.horizontal_sum(a) == 12);
    small := V.min(a, b);
    large := V.max(a, cast(s32) 7);

    for 0..3 assert(small.data[it] == 3 && large.data[it] == 7);

    mask := b > cast(s32) 4;

    assert(V.get_mask(mask) == 15);
    for mask.data assert(it == 1);

    a.data = .[-1, 0, 1, 2];
    assert(V.get_mask(a) == 13);
    mask = a >= cast(s32) 1;
    assert(V.get_mask(mask) == 12);
    mask = a == a;
    assert(V.get_mask(mask) == 15);
    mask = a != b;
    assert(V.get_mask(mask) == 15);
    assert(V.get_mask(a < b) == 15);
    assert(V.get_mask(a <= b) == 15);
    assert(V.get_mask(b >= a) == 15);
    assert(V.get_mask(b > a) == 15);
    assert(V.get_mask(a == cast(s32) 0) == 2);
    assert(V.get_mask(a < cast(s32) 0) == 1);
    assert(V.get_mask(a <= cast(s32) 0) == 3);
    assert(V.get_mask(a != cast(s32) 0) == 13);
    scalar = b - cast(s32) 2;
    for scalar.data assert(it == 3);

    scalar = b / cast(s32) 5;
    for scalar.data assert(it == 1);

    small = V.min(b, cast(s32) 2);
    large = V.max(a, b);
    for 0..3 assert(small.data[it] == 2 && large.data[it] == 5);

    output: [18] u8;
    output[0] = 0xa5;
    output[17] = 0x5a;

    V.store(output.data + 1, a);
    values: [4] s32;

    memcpy(values.data, output.data + 1, 16);
    for 0..3 assert(values[it] == a.data[it]);

    contiguous := V.load(output.data + 1, 4, s32);
    for 0..3 assert(contiguous.data[it] == a.data[it]);

    offsets := V.set(4, cast(s32) 0);
    offsets.data = .[0, 8, 0, 12];
    gathered := V.load_from_offsets(output.data + 1, offsets, s32);

    assert(gathered.data[0] == -1 && gathered.data[1] == 1 && gathered.data[2] == -1 && gathered.data[3] == 2);
    scattered := V.set(4, cast(s32) 0);
    scattered.data = .[11, 22, 33, 44];
    offsets.data = .[12, 4, 4, 0];
    V.store_with_offsets(scattered, output.data + 1, offsets);
    contiguous = V.load(output.data + 1, 4, s32);
    assert(contiguous.data[0] == 44 && contiguous.data[1] == 33 && contiguous.data[2] == 1 && contiguous.data[3] == 11);
    assert(output[0] == 0xa5 && output[17] == 0x5a);
    offsets.data = .[-4, 0, -12, -4];
    gathered = V.load_from_offsets(output.data + 13, offsets, s32);
    assert(gathered.data[0] == 1 && gathered.data[1] == 11 && gathered.data[2] == 44 && gathered.data[3] == 1);
    indices := V.get_looping_indices(4, s32);
    stepped := V.get_looping_indices(4, s32, 3);
    reversed := V.get_looping_indices(4, s32, -2);

    for 0..3 assert(indices.data[it] == it && stepped.data[it] == it * 3 && reversed.data[it] == it * -2);

    pairs := V.horizontal_add(a, b);

    assert(pairs.data[0] == -1 && pairs.data[1] == 10 && pairs.data[2] == 3 && pairs.data[3] == 10);

    wrapped := V.set(4, cast(u8) 255) + cast(u8) 1;
    for wrapped.data assert(it == 0);

    floats := V.set(2, cast(float64) 2.5) * cast(float64) 4;

    assert(V.horizontal_sum(floats) == 20);
    assert(V.get_mask(floats > cast(float64) 9) == 3);
    float_mask := floats > cast(float64) 9;
    for float_mask.data assert(it == 1.0);

    truth := V.set(4, cast(float64) 0);
    truth.data = .[0, -0.0, 1.5, -1.5];
    assert(V.get_mask(truth) == 12);
    negative := V.set(2, cast(float64) -1);

    assert(V.get_mask(negative) == 3);
    signed_wrap := V.set(2, cast(s8) 127) + cast(s8) 1;
    for signed_wrap.data assert(it == -128);

    wide := V.set(64, cast(u8) 1);

    assert(V.get_mask(wide == cast(u8) 1) == -1);
    union_value: V.Simd_128;
    union_value._u64.data[0] = 0x0807060504030201;
    for 0..7 assert(union_value._u8.data[it] == it + 1);

    assert((cast(*Type_Info_Struct) type_info(V.Simd_128)).name == "Simd_128");
    assert((cast(*Type_Info_Struct) type_info(V.SIMD_Array(4, s32))).name == "SIMD_Array");
    print("ok\n");
}
