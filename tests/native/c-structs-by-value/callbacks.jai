// Jai #c_call procedures taking and returning C structs by value, called from C (interpreter and native).
#import "Basic";
structs :: #library "libstructs";
#load "types.jai";

combine :: (v: V2, b: Big, s: Small) -> V2 #c_call {
    return .{ v.x + xx b.a + xx s.a, v.y + xx b.d + xx s.b };
}
scale :: (b: Big, k: s32) -> Big #c_call {
    return .{ b.a * k, b.b * k, b.c * k, b.d * k };
}
bump :: (s: Small) -> Small #c_call {
    return .{ s.a + 1, s.b + 1 };
}

many_back :: (a: s64, b: s64, c: s64, d: s64, e: s64, f: s64, g: s64, l: Large,
    d1: float64, d2: float64, d3: float64, d4: float64, d5: float64, d6: float64, d7: float64, d8: float64, d9: float64) -> float64 #c_call {
    return xx (a + b + c + d + e + f + g + l.a + l.b + l.c) + d1 + d2 + d3 + d4 + d5 + d6 + d7 + d8 + d9 * 100;
}

mixed_back :: (a: float64, b: s32, c: float, d: s64, e: float64, g: s16) -> Mixed #c_call {
    return .{ b + d + g, a + c + e };
}
v2_back :: (a: float, b: s64, c: float64, d: s32) -> V2 #c_call {
    return .{ a + xx b, xx (c + xx d) };
}

main :: () {
    print("% % %\n", apply(combine, .{ 1, 2 }, .{ 10, 20, 30, 40 }, .{ 100, 5 }), apply_big(scale), apply_small(bump));
    print("%\n", apply_many(many_back));
    print("% %\n", apply_mixed(mixed_back), apply_mixed_v2(v2_back));
}
