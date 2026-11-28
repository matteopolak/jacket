// Portable fixed-width vectors.
//
// ```jai
// SIMD :: #import "Extensions/SIMD";
// using SIMD;
//
// text := "hello, wide world!";
//
// // Where is the first space in these 16 bytes?
// chunk := load(u8x16, text);
// spaces := eq(chunk, #char " ");
// assert(any(spaces) && first_true(spaces) == 6);
// ```
//
// A Vector(T, N) is N lanes of T, at most 64 bytes. Every operation works lane by lane, and its
// result is the same on every target: the interpreter, x86-64, arm64 and wasm. Compiled with
// optimization, the operations become the target's vector instructions. Integer arithmetic wraps.
// Comparisons give a Mask, which selects lanes, reduces to a bool, or becomes a bit per lane.

// The vector width code should aim for when it can choose: 16 bytes is SSE2, NEON and wasm
// simd128, the widest every target of those has. Set 32 when building for AVX2.
#module_parameters(VECTOR_BYTES := 16);

// `N` lanes of `T`, at most 64 bytes in all. `T` is an integer or float type, and `N` a power of
// two. Use the named types (`u8x16`, `f32x4`) or `Vector(T, N)` directly; read and write lanes
// with `vector[i]`.
//
// ```jai
// v := indices(u32x4);
// v[0] = 9;
// assert(v[0] == 9 && v[3] == 3);
// ```
Vector :: struct(T: Type, N: int) #modify {
    return lanes_are_valid(T, N);
} {
    lanes: [N] T;
}

// One lane per lane of Vector(T, N): all ones where a comparison held, all zeros where it didn't.
// The lanes are as wide as T, so masks and vectors line up the way the hardware wants.
Mask :: struct(T: Type, N: int) #modify {
    return lanes_are_valid(T, N);
} {
    lanes: [N] #run lane_bits(T);
}

u8x16 :: Vector(u8, 16);
u8x32 :: Vector(u8, 32);
u8x64 :: Vector(u8, 64);
s8x16 :: Vector(s8, 16);
s8x32 :: Vector(s8, 32);
u16x8 :: Vector(u16, 8);
u16x16 :: Vector(u16, 16);
s16x8 :: Vector(s16, 8);
s16x16 :: Vector(s16, 16);
u32x4 :: Vector(u32, 4);
u32x8 :: Vector(u32, 8);
s32x4 :: Vector(s32, 4);
s32x8 :: Vector(s32, 8);
u64x2 :: Vector(u64, 2);
u64x4 :: Vector(u64, 4);
s64x2 :: Vector(s64, 2);
s64x4 :: Vector(s64, 4);
f32x4 :: Vector(float32, 4);
f32x8 :: Vector(float32, 8);
f64x2 :: Vector(float64, 2);
f64x4 :: Vector(float64, 4);

// Making and moving vectors

// Every lane set to `value`.
splat :: ($V: Type, value: V.T) -> V #modify {
    return is_vector(V);
} {
    result: V = ---;
    for 0..V.N - 1 result.lanes[it] = value;
    return result;
}

// Lane i holds i.
indices :: ($V: Type) -> V #modify {
    return is_vector(V);
} {
    result: V = ---;
    for 0..V.N - 1 result.lanes[it] = cast(V.T) it;
    return result;
}

// V.N values starting at `source`, which needs no particular alignment.
load :: inline ($V: Type, source: *V.T) -> V #modify {
    return is_vector(V);
} {
    result: V = ---;
    memcpy(*result, source, size_of(V));
    return result;
}

// V.N items starting at `items[start]`, which must all exist.
load :: inline ($V: Type, items: [] V.T, start := 0) -> V #modify {
    return is_vector(V);
} {
    assert(start >= 0 && start + V.N <= items.count, "SIMD: load of % lanes at % reads past the % items", V.N, start, items.count);
    return load(V, items.data + start);
}

// V.N bytes of `text` starting at `start`, which must all exist.
load :: inline ($V: Type, text: string, start := 0) -> V #modify {
    return is_byte_vector(V);
} {
    assert(start >= 0 && start + V.N <= text.count, "SIMD: load of % bytes at % reads past the end of a %-byte string", V.N, start, text.count);
    return load(V, text.data + start);
}

// The first `count` lanes from `source` (0 <= count <= V.N), the rest `fill`. For the tail of a
// buffer that has fewer than V.N items left.
load_partial :: ($V: Type, source: *V.T, count: int, fill: V.T = 0) -> V #modify {
    return is_vector(V);
} {
    assert(count >= 0 && count <= V.N, "SIMD: load_partial of % lanes into a vector of %", count, V.N);
    result: V;

    // A new vector is all zero already, so only a lane value that is not integer zero needs spreading
    // (a float zero may be negative).
    #if is_integer(V.T) {
        if fill != 0 result = splat(V, fill);
    } else result = splat(V, fill);

    memcpy(*result, source, count * size_of(V.T));

    return result;
}

// Writes the N lanes to `dest`, which needs no particular alignment.
//
// ```jai
// out: [4] s32;
// store(splat(s32x4, 7), out.data);
// assert(out[3] == 7);
// ```
store :: inline (vector: Vector($T, $N), dest: *T) {
    memcpy(dest, *vector, size_of(type_of(vector)));
}

// Writes the N lanes to `items[start]` onward, which must all exist; asserts otherwise.
//
// ```jai
// out: [8] u16;
// store(splat(u16x8, 1), out, 0);
// ```
store :: inline (vector: Vector($T, $N), items: [] T, start := 0) {
    assert(start >= 0 && start + N <= items.count, "SIMD: store of % lanes at % writes past the % items", N, start, items.count);
    store(vector, items.data + start);
}

// Lane `index`. An index outside `0..N - 1` is a bounds error, as for an array.
operator [] :: inline (vector: Vector($T, $N), index: int) -> T {
    return vector.lanes[index];
}

// Sets lane `index` to `value`.
operator []= :: inline (vector: *Vector($T, $N), index: int, value: T) {
    vector.lanes[index] = value;
}

// Arithmetic. Integer lanes wrap; use saturating_add/saturating_sub to clamp instead.
operator + :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #no_aoc {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] + b.lanes[it];
    return result;
}

// `a - b` per lane; integer lanes wrap.
operator - :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #no_aoc {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] - b.lanes[it];
    return result;
}

// `a * b` per lane; integer lanes keep the low bits.
operator * :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #no_aoc {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] * b.lanes[it];
    return result;
}

// Integer division by a zero lane is an error, as for scalars.
operator / :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] / b.lanes[it];
    return result;
}

// Each lane negated. The most negative integer stays itself.
operator - :: inline (a: Vector($T, $N)) -> Vector(T, N) #no_aoc {
    result: Vector(T, N) = ---;

    #if is_float(T) {
        for 0..N - 1 result.lanes[it] = -a.lanes[it];
    } else {
        for 0..N - 1 result.lanes[it] = 0 - a.lanes[it];
    }

    return result;
}

// A scalar on either side applies to every lane.
operator + :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) #symmetric {
    return a + splat(Vector(T, N), b);
}

// Every lane times the scalar `b`, on either side.
operator * :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) #symmetric {
    return a * splat(Vector(T, N), b);
}

// Every lane minus the scalar `b`.
operator - :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) {
    return a - splat(Vector(T, N), b);
}

// The scalar `a` minus every lane.
operator - :: inline (a: T, b: Vector($T, $N)) -> Vector(T, N) {
    return splat(Vector(T, N), a) - b;
}

// Every lane divided by the scalar `b`.
operator / :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) {
    return a / splat(Vector(T, N), b);
}

// The scalar `a` divided by every lane.
operator / :: inline (a: T, b: Vector($T, $N)) -> Vector(T, N) {
    return splat(Vector(T, N), a) / b;
}

// The smaller of `a` and `b` in each lane. For floats, a NaN in `a` gives `b`'s lane.
//
// ```jai
// a := s32x4.{ .[1, 5, -3, 0] };
// assert(min(a, splat(s32x4, 0))[2] == -3);
// assert(max(a, splat(s32x4, 0))[1] == 5);
// ```
min :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = ifx a.lanes[it] < b.lanes[it] then a.lanes[it] else b.lanes[it];
    return result;
}

// The larger of `a` and `b` in each lane. For floats, a NaN in `a` gives `b`'s lane.
max :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = ifx a.lanes[it] > b.lanes[it] then a.lanes[it] else b.lanes[it];
    return result;
}

// The absolute value of each lane. The most negative integer stays itself, as it has no
// positive counterpart.
abs :: inline (a: Vector($T, $N)) -> Vector(T, N) #no_aoc #modify {
    return is_signed_or_float(T), "abs needs signed or float lanes";
} {
    result: Vector(T, N) = ---;

    #if is_float(T) {
        for 0..N - 1 result.lanes[it] = ifx a.lanes[it] < 0 then -a.lanes[it] else a.lanes[it];
    } else {
        for 0..N - 1 result.lanes[it] = ifx a.lanes[it] < 0 then 0 - a.lanes[it] else a.lanes[it];
    }

    return result;
}

// a + b per lane, clamped to T's range instead of wrapping.
saturating_add :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #no_aoc #modify {
    return is_integer(T), "saturating_add needs integer lanes";
} {
    result: Vector(T, N) = ---;

    for 0..N - 1 {
        x, y := a.lanes[it], b.lanes[it];
        sum := x + y;

        #if is_signed(T) {
            // Overflow happened when both inputs share a sign the sum lacks.
            if (x >= 0) == (y >= 0) && (sum >= 0) != (x >= 0) sum = ifx x >= 0 then signed_max(T) else ~signed_max(T);
        } else {
            if sum < x sum = ~cast(T) 0;
        }

        result.lanes[it] = sum;
    }

    return result;
}

// a - b per lane, clamped to T's range instead of wrapping.
saturating_sub :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #no_aoc #modify {
    return is_integer(T), "saturating_sub needs integer lanes";
} {
    result: Vector(T, N) = ---;

    for 0..N - 1 {
        x, y := a.lanes[it], b.lanes[it];
        difference := x - y;

        #if is_signed(T) {
            if (x >= 0) != (y >= 0) && (difference >= 0) != (x >= 0) difference = ifx x >= 0 then signed_max(T) else ~signed_max(T);
        } else {
            if y > x difference = 0;
        }

        result.lanes[it] = difference;
    }

    return result;
}

// The square root of each float lane; a negative lane gives NaN.
//
// ```jai
// assert(sqrt(f32x4.{ .[4, 9, 16, 25] })[1] == 3);
// ```
sqrt :: inline (a: Vector($T, $N)) -> Vector(T, N) #modify {
    return is_float(T), "sqrt needs float lanes";
} {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = sqrt(a.lanes[it]);
    return result;
}

// Bitwise operations, on integer lanes.
operator & :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #modify {
    return is_integer(T), "& needs integer lanes";
} {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] & b.lanes[it];
    return result;
}

// Bitwise or of each pair of integer lanes.
operator | :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #modify {
    return is_integer(T), "| needs integer lanes";
} {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] | b.lanes[it];
    return result;
}

// Bitwise exclusive or of each pair of integer lanes.
operator ^ :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Vector(T, N) #modify {
    return is_integer(T), "^ needs integer lanes";
} {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] ^ b.lanes[it];
    return result;
}

// Each integer lane with every bit flipped.
operator ~ :: inline (a: Vector($T, $N)) -> Vector(T, N) #modify {
    return is_integer(T), "~ needs integer lanes";
} {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = ~a.lanes[it];
    return result;
}

// Every integer lane and the scalar `b`, on either side.
operator & :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) #symmetric {
    return a & splat(Vector(T, N), b);
}

// Every integer lane or the scalar `b`, on either side.
operator | :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) #symmetric {
    return a | splat(Vector(T, N), b);
}

// Every integer lane exclusive-or the scalar `b`, on either side.
operator ^ :: inline (a: Vector($T, $N), b: T) -> Vector(T, N) #symmetric {
    return a ^ splat(Vector(T, N), b);
}

// Each lane shifted by `count` bits, which must be less than the lane's width. >> keeps the sign
// of signed lanes.
operator << :: inline (a: Vector($T, $N), count: int) -> Vector(T, N) #modify {
    return is_integer(T), "<< needs integer lanes";
} {
    // The count wraps at the lane width, as wasm's shifts do, so every count is defined.
    bits := count & (size_of(T) * 8 - 1);
    result: Vector(T, N) = ---;

    for 0..N - 1 result.lanes[it] = a.lanes[it] << bits;

    return result;
}

// Each lane shifted right by `count` bits, wrapped at the lane width; signed lanes keep their sign.
operator >> :: inline (a: Vector($T, $N), count: int) -> Vector(T, N) #modify {
    return is_integer(T), ">> needs integer lanes";
} {
    // The count wraps at the lane width, as wasm's shifts do, so every count is defined.
    bits := count & (size_of(T) * 8 - 1);
    result: Vector(T, N) = ---;

    for 0..N - 1 result.lanes[it] = a.lanes[it] >> bits;

    return result;
}

// Comparisons, each giving a Mask. Float lanes compare as scalars do, so NaN is unequal to
// everything (ne holds, the others don't).
eq :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Mask(T, N) {
    #insert #run compare_body("==");
}

// Lanes where `a != b`, including every NaN lane.
ne :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Mask(T, N) {
    #insert #run compare_body("!=");
}

// Lanes where `a < b`.
lt :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Mask(T, N) {
    #insert #run compare_body("<");
}

// Lanes where `a <= b`.
le :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Mask(T, N) {
    #insert #run compare_body("<=");
}

// Lanes where `a > b`.
gt :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Mask(T, N) {
    #insert #run compare_body(">");
}

// Lanes where `a >= b`.
ge :: inline (a: Vector($T, $N), b: Vector(T, N)) -> Mask(T, N) {
    #insert #run compare_body(">=");
}

// Lanes equal to the scalar `b`.
//
// ```jai
// m := eq(u8x16.{ .[1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1] }, 1);
// assert(count_true(m) == 3 && first_true(m) == 0 && last_true(m) == 15);
// ```
eq :: inline (a: Vector($T, $N), b: T) -> Mask(T, N) {
    return eq(a, splat(Vector(T, N), b));
}

// Lanes not equal to the scalar `b`.
ne :: inline (a: Vector($T, $N), b: T) -> Mask(T, N) {
    return ne(a, splat(Vector(T, N), b));
}

// Lanes less than the scalar `b`.
lt :: inline (a: Vector($T, $N), b: T) -> Mask(T, N) {
    return lt(a, splat(Vector(T, N), b));
}

// Lanes at most the scalar `b`.
le :: inline (a: Vector($T, $N), b: T) -> Mask(T, N) {
    return le(a, splat(Vector(T, N), b));
}

// Lanes greater than the scalar `b`.
gt :: inline (a: Vector($T, $N), b: T) -> Mask(T, N) {
    return gt(a, splat(Vector(T, N), b));
}

// Lanes at least the scalar `b`.
ge :: inline (a: Vector($T, $N), b: T) -> Mask(T, N) {
    return ge(a, splat(Vector(T, N), b));
}

// Masks

// Lanes set in both masks.
operator & :: inline (a: Mask($T, $N), b: Mask(T, N)) -> Mask(T, N) {
    result: Mask(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] & b.lanes[it];
    return result;
}

// Lanes set in either mask.
operator | :: inline (a: Mask($T, $N), b: Mask(T, N)) -> Mask(T, N) {
    result: Mask(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] | b.lanes[it];
    return result;
}

// Lanes set in exactly one of the masks.
operator ^ :: inline (a: Mask($T, $N), b: Mask(T, N)) -> Mask(T, N) {
    result: Mask(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[it] ^ b.lanes[it];
    return result;
}

// The lanes not set in `a`.
operator ~ :: inline (a: Mask($T, $N)) -> Mask(T, N) {
    result: Mask(T, N) = ---;
    for 0..N - 1 result.lanes[it] = ~a.lanes[it];
    return result;
}

// Whether lane `index` is set.
operator [] :: inline (mask: Mask($T, $N), index: int) -> bool {
    return mask.lanes[index] != 0;
}

// Bit i set where lane i is set.
to_bits :: inline (mask: Mask($T, $N)) -> u64 {
    #if size_of(T) == 1 && N >= 16 {
        return sign_mask(cast(*u8) *mask.lanes[0], N);
    } else {
        bits: u64;
        for 0..N - 1 bits |= (cast(u64) (mask.lanes[it] & 1)) << it;
        return bits;
    }
}

// A mask whose lane i is set where bit i of `bits` is.
mask_from_bits :: ($M: Type, bits: u64) -> M #modify {
    return is_mask(M);
} {
    result: M = ---;
    for 0..M.N - 1 result.lanes[it] = ifx bits & ((cast(u64) 1) << it) then ~cast(type_of(result.lanes[0])) 0 else 0;
    return result;
}

// Whether at least one lane is set.
//
// ```jai
// m := gt(s32x4.{ .[1, 2, 3, 4] }, 2);
// assert(any(m) && !all(m) && !none(m));
// ```
any :: inline (mask: Mask($T, $N)) -> bool {
    return reduce_or_lanes(mask) != 0;
}

// Whether every lane is set.
all :: inline (mask: Mask($T, $N)) -> bool {
    return reduce_and_lanes(mask) != 0;
}

// Whether no lane is set.
none :: inline (mask: Mask($T, $N)) -> bool {
    return reduce_or_lanes(mask) == 0;
}

// How many lanes are set.
count_true :: inline (mask: Mask($T, $N)) -> int {
    return bit_count(to_bits(mask));
}

// The lowest set lane, or -1 if none is.
first_true :: inline (mask: Mask($T, $N)) -> int {
    bits := to_bits(mask);
    if bits == 0 return -1;
    return trailing_zeros(bits);
}

// The highest set lane, or -1 if none is.
last_true :: inline (mask: Mask($T, $N)) -> int {
    bits := to_bits(mask);
    if bits == 0 return -1;
    return 63 - leading_zeros(bits);
}

// Lane i from `a` where the mask is set, from `b` where it isn't.
select :: inline (mask: Mask($T, $N), a: Vector(T, N), b: Vector(T, N)) -> Vector(T, N) {
    result: Vector(T, N) = ---;

    #if is_float(T) {
        for 0..N - 1 result.lanes[it] = ifx mask.lanes[it] then a.lanes[it] else b.lanes[it];
    } else {
        for 0..N - 1 {
            m := cast,no_check(T) mask.lanes[it];
            result.lanes[it] = (a.lanes[it] & m) | (b.lanes[it] & ~m);
        }
    }

    return result;
}

// Reductions: all lanes combined into one value.
reduce_add :: inline (a: Vector($T, $N)) -> T #no_aoc {
    total: T = 0;
    for 0..N - 1 total += a.lanes[it];
    return total;
}

// The smallest lane.
//
// ```jai
// v := s32x4.{ .[4, -1, 7, 2] };
// assert(reduce_min(v) == -1 && reduce_max(v) == 7 && reduce_add(v) == 12);
// ```
reduce_min :: inline (a: Vector($T, $N)) -> T {
    result := a.lanes[0];
    for 1..N - 1 if a.lanes[it] < result result = a.lanes[it];
    return result;
}

// The largest lane.
reduce_max :: inline (a: Vector($T, $N)) -> T {
    result := a.lanes[0];
    for 1..N - 1 if a.lanes[it] > result result = a.lanes[it];
    return result;
}

// Every integer lane combined with `&`.
reduce_and :: inline (a: Vector($T, $N)) -> T #modify {
    return is_integer(T), "reduce_and needs integer lanes";
} {
    result := a.lanes[0];
    for 1..N - 1 result &= a.lanes[it];
    return result;
}

// Every integer lane combined with `|`.
reduce_or :: inline (a: Vector($T, $N)) -> T #modify {
    return is_integer(T), "reduce_or needs integer lanes";
} {
    result := a.lanes[0];
    for 1..N - 1 result |= a.lanes[it];
    return result;
}

// Every integer lane combined with `^`.
reduce_xor :: inline (a: Vector($T, $N)) -> T #modify {
    return is_integer(T), "reduce_xor needs integer lanes";
} {
    result := a.lanes[0];
    for 1..N - 1 result ^= a.lanes[it];
    return result;
}

// Rearranging lanes

// Lane i is a[order[i]]. The order is a constant, so this becomes one shuffle instruction.
shuffle :: inline (a: Vector($T, $N), $order: [] int) -> Vector(T, N) {
    #assert(order.count == N && order_is_valid(order, N), "SIMD: shuffle order needs N indices from 0 to N - 1");
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[order[it]];
    return result;
}

// Lane i is lane order[i] of a followed by b: indices below N pick from a, the rest from b.
shuffle :: inline (a: Vector($T, $N), b: Vector(T, N), $order: [] int) -> Vector(T, N) {
    #assert(order.count == N && order_is_valid(order, 2 * N), "SIMD: shuffle order needs N indices from 0 to 2N - 1");
    result: Vector(T, N) = ---;

    for 0..N - 1 {
        index := order[it];
        result.lanes[it] = ifx index < N then a.lanes[index] else b.lanes[index - N];
    }

    return result;
}

// N lanes of a followed by b, starting at lane `start`: window(a, b, 0) is a, window(a, b, N) is
// b. With `a` the previous block of a stream and `b` the current one, window(a, b, N - k) is the
// current block moved k lanes later, with the end of the previous one in front.
window :: inline (a: Vector($T, $N), b: Vector(T, N), $start: int) -> Vector(T, N) {
    #assert(start >= 0 && start <= N, "SIMD: window start must be from 0 to N");
    result: Vector(T, N) = ---;

    for 0..N - 1 {
        index := start + it;
        result.lanes[it] = ifx index < N then a.lanes[index] else b.lanes[index - N];
    }

    return result;
}

// The lanes in the opposite order.
//
// ```jai
// assert(reverse(indices(u32x4))[0] == 3);
// ```
reverse :: inline (a: Vector($T, $N)) -> Vector(T, N) {
    result: Vector(T, N) = ---;
    for 0..N - 1 result.lanes[it] = a.lanes[N - 1 - it];
    return result;
}

// A byte table lookup: lane i is table[indices[i]] when that index is below 16, else 0. This is
// one instruction on SSSE3, NEON and wasm simd128, and the base of fast byte classification:
// split each byte into nibbles and look both up.
lookup :: inline (table: u8x16, indices: Vector(u8, $N)) -> Vector(u8, N) {
    result: Vector(u8, N) = ---;

    #if N >= 16 {
        byte_lookup(*result.lanes[0], *table.lanes[0], *indices.lanes[0], N);
        return result;
    }

    for 0..N - 1 {
        index := indices.lanes[it];
        result.lanes[it] = ifx index < 16 then table.lanes[index & 15] else 0;
    }

    return result;
}

// Changing shape

// Each lane converted with a cast: floats to integers truncate toward zero, integers to narrower
// integers keep the low bits.
convert :: ($V: Type, a: Vector($T, V.N)) -> V #modify {
    return is_vector(V);
} {
    result: V = ---;

    #if is_float(V.T) || is_float(T) {
        for 0..V.N - 1 result.lanes[it] = cast(V.T) a.lanes[it];
    } else {
        for 0..V.N - 1 result.lanes[it] = cast,trunc(V.T) a.lanes[it];
    }

    return result;
}

// The same bytes seen as another vector type of the same size.
bitcast :: inline ($V: Type, a: Vector($T, $N)) -> V #modify {
    return is_vector(V);
} {
    #assert(size_of(V) == N * size_of(T), "SIMD: bitcast must keep the size");
    result: V = ---;
    memcpy(*result, *a, size_of(V));
    return result;
}

// A mask's lanes as a vector of the same size: all-ones lanes and zero lanes.
bitcast :: inline ($V: Type, mask: Mask($T, $N)) -> V #modify {
    return is_vector(V);
} {
    #assert(size_of(V) == size_of(Mask(T, N)), "SIMD: bitcast must keep the size");
    result: V = ---;
    memcpy(*result, *mask, size_of(V));
    return result;
}

// The first N / 2 lanes; [combine] puts the halves back together.
//
// ```jai
// v := indices(u16x16);
// assert(low_half(v)[0] == 0 && high_half(v)[0] == 8);
// ```
low_half :: inline (a: Vector($T, $N)) -> Vector(T, N / 2) {
    result: Vector(T, N / 2) = ---;
    for 0..N / 2 - 1 result.lanes[it] = a.lanes[it];
    return result;
}

// The last N / 2 lanes.
high_half :: inline (a: Vector($T, $N)) -> Vector(T, N / 2) {
    result: Vector(T, N / 2) = ---;
    for 0..N / 2 - 1 result.lanes[it] = a.lanes[N / 2 + it];
    return result;
}

// `low` then `high`, as one vector of twice the lanes.
combine :: inline (low: Vector($T, $N), high: Vector(T, N)) -> Vector(T, 2 * N) {
    result: Vector(T, 2 * N) = ---;

    for 0..N - 1 {
        result.lanes[it] = low.lanes[it];
        result.lanes[N + it] = high.lanes[it];
    }

    return result;
}

// Bits of a u64, for the masks above.
bit_count :: inline (x: u64) -> int {
    result: u64;
    #asm { popcnt.q result, x; }
    return cast(int) result;
}

// The number of 0 bits below the lowest 1 bit; 64 for 0.
trailing_zeros :: inline (x: u64) -> int {
    result: u64;
    #asm { tzcnt.q result, x; }
    return cast(int) result;
}

// The number of 0 bits above the highest 1 bit; 64 for 0.
leading_zeros :: inline (x: u64) -> int {
    result: u64;
    #asm { lzcnt.q result, x; }
    return cast(int) result;
}

// Choosing code by processor. A procedure compiled for a feature (`#target_features "avx2"`) is only
// safe to call where the processor has it; these say what it has and pick the best version once.

Cpu_Features :: Cpu.Cpu_Features;
Vector_Tier :: Cpu.Vector_Tier;
cpu_features :: Cpu.cpu_features;
best_vector_tier :: Cpu.best_vector_tier;
limit_vector_tier :: Cpu.limit_vector_tier;
pick_target :: Cpu.pick_target;

#scope_module

Cpu :: #import "internal/cpu";
#import "Basic";

// The square root instruction, for the lanes of sqrt. Declared here so that importing SIMD does not
// load all of Math.
sqrt :: (x: float32) -> float32 #intrinsic;
sqrt :: (x: float64) -> float64 #intrinsic;

// Compiler intrinsics: a 16-byte table lookup per 16-byte block (indices of 16 and up give 0), and
// the top bit of each byte packed into an integer. See docs/language/intrinsics.md.
byte_lookup :: (dest: *u8, table: *u8, indices: *u8, width: s64) #intrinsic;
sign_mask :: (source: *u8, width: s64) -> u64 #intrinsic;

lanes_are_valid :: (T: Type, N: int) -> bool, string {
    info := cast(*Type_Info) T;
    if info.type != .INTEGER && info.type != .FLOAT {
        return false, tprint("SIMD: lanes must be integers or floats, not %", T);
    }

    if N <= 0 || (N & (N - 1)) != 0 return false, tprint("SIMD: the lane count must be a power of two, not %", N);
    if N * info.runtime_size > 64 return false, tprint("SIMD: % lanes of % make % bytes; a vector holds at most 64", N, T, N * info.runtime_size);

    return true, "";
}

lane_bits :: (T: Type) -> Type {
    size := (cast(*Type_Info) T).runtime_size;
    if size == 1 return u8;
    if size == 2 return u16;
    if size == 4 return u32;

    return u64;
}

is_vector :: (V: Type) -> bool, string {
    info := cast(*Type_Info) V;
    if info.type == .STRUCT && (cast(*Type_Info_Struct) info).polymorph_source_struct == type_info(Vector) return true, "";
    return false, tprint("SIMD: expected a Vector type, not %", V);
}

is_byte_vector :: (V: Type) -> bool, string {
    ok, message := is_vector(V);
    if !ok return false, message;

    lanes := cast(*Type_Info_Array) (cast(*Type_Info_Struct) V).members[0].type;
    if lanes.element_type == type_info(u8) return true, "";

    return false, tprint("SIMD: a string loads into a vector of u8, not %", V);
}

is_mask :: (M: Type) -> bool, string {
    info := cast(*Type_Info) M;
    if info.type == .STRUCT && (cast(*Type_Info_Struct) info).polymorph_source_struct == type_info(Mask) return true, "";
    return false, tprint("SIMD: expected a Mask type, not %", M);
}

signed_max :: inline ($T: Type) -> T {
    return cast,no_check(T) (((cast(u64) 1) << (size_of(T) * 8 - 1)) - 1);
}

is_float :: (T: Type) -> bool {
    return (cast(*Type_Info) T).type == .FLOAT;
}

is_integer :: (T: Type) -> bool {
    return (cast(*Type_Info) T).type == .INTEGER;
}

is_signed :: (T: Type) -> bool {
    info := cast(*Type_Info) T;
    return info.type == .INTEGER && (cast(*Type_Info_Integer) info).signed;
}

is_signed_or_float :: (T: Type) -> bool {
    return is_signed(T) || is_float(T);
}

order_is_valid :: (order: [] int, limit: int) -> bool {
    for order if it < 0 || it >= limit return false;
    return true;
}

compare_body :: (operator: string) -> string {
    return tprint(#string JAI
    result: Mask(T, N) = ---;
    for 0..N - 1 result.lanes[it] = ifx a.lanes[it] %1 b.lanes[it] then ~cast(type_of(result.lanes[0])) 0 else 0;
    return result;
JAI, operator);
}

// A mask's lanes are all ones or all zeros, so its 64-bit words can stand in for the lanes:
// fewer, wider operations than one per lane.
reduce_or_lanes :: inline (mask: Mask($T, $N)) -> u64 {
    result: u64;

    #if N * size_of(T) >= 8 {
        words: [N * size_of(T) / 8] u64 = ---;
        memcpy(words.data, *mask, size_of(type_of(words)));
        for words result |= it;
    } else {
        for 0..N - 1 result |= mask.lanes[it];
    }

    return result;
}

reduce_and_lanes :: inline (mask: Mask($T, $N)) -> u64 {
    result := ~cast(u64) 0;

    #if N * size_of(T) >= 8 {
        words: [N * size_of(T) / 8] u64 = ---;
        memcpy(words.data, *mask, size_of(type_of(words)));
        for words result &= it;
        if result != ~cast(u64) 0 result = 0;
    } else {
        for 0..N - 1 result &= mask.lanes[it];
    }

    return result;
}
