V2 :: struct {
    x, y: float;
}

V3 :: struct {
    x, y, z: float;
}

Big :: struct {
    a, b, c, d: float64;
}

Small :: struct {
    a: s32;
    b: u8;
}

Mixed :: struct {
    a: s64;
    b: float64;
}

Mixed2 :: struct {
    b: float64;
    a: s64;
}

Large :: struct {
    a, b, c: s64;
}

add_v2 :: (p: V2, q: V2) -> V2 #foreign structs;
scale_v3 :: (v: V3, k: float) -> V3 #foreign structs;
make_big :: (base: float64) -> Big #foreign structs;
sum_big :: (b: Big) -> float64 #foreign structs;
make_small :: (a: s32, b: u8) -> Small #foreign structs;
swap_mixed :: (m: Mixed2) -> Mixed #foreign structs;
add_large :: (x: Large, y: Large) -> Large #foreign structs;
apply :: (f: (V2, Big, Small) -> V2 #c_call, v: V2, b: Big, s: Small) -> V2 #foreign structs;
apply_big :: (f: (Big, s32) -> Big #c_call) -> Big #foreign structs;
apply_small :: (f: (Small) -> Small #c_call) -> Small #foreign structs;

many :: (a: s64, b: s64, c: s64, d: s64, e: s64, f: s64, g: s64, l: Large,
    d1: float64, d2: float64, d3: float64, d4: float64, d5: float64, d6: float64, d7: float64, d8: float64, d9: float64) -> float64 #foreign structs;

apply_many :: (f: (a: s64, b: s64, c: s64, d: s64, e: s64, f: s64, g: s64, l: Large,
    d1: float64, d2: float64, d3: float64, d4: float64, d5: float64, d6: float64, d7: float64, d8: float64, d9: float64) -> float64 #c_call) -> float64 #foreign structs;

apply_mixed :: (f: (a: float64, b: s32, c: float, d: s64, e: float64, g: s16) -> Mixed #c_call) -> Mixed #foreign structs;
apply_mixed_v2 :: (f: (a: float, b: s64, c: float64, d: s32) -> V2 #c_call) -> V2 #foreign structs;
narrow_tail :: (a: s64, b: s64, c: s64, d: s64, e: s64, f: s64, g: s64, h: s64,
    d1: float64, d2: float64, d3: float64, d4: float64, d5: float64, d6: float64, d7: float64, d8: float64,
    i: s8, j: s16, k: u8, l: s32, m: float, n: s16) -> s64 #foreign structs;
apply_narrow_tail :: (f: (a: s64, b: s64, c: s64, d: s64, e: s64, f: s64, g: s64, h: s64,
    d1: float64, d2: float64, d3: float64, d4: float64, d5: float64, d6: float64, d7: float64, d8: float64,
    i: s8, j: s16, k: u8, l: s32, m: float, n: s16) -> s64 #c_call) -> s64 #foreign structs;
