#import "Basic";
Vec :: struct (T: Type, N: int) { lanes: [N] T; }
f :: (a: Vec($T, $N)) -> int #modify { return N > 0; } { return N; }
g :: (a: [$C] int) -> int #modify { return C > 0; } { return C; }
h :: ($V: Type, a: Vec($T, $N)) -> V #modify { return N > 0; } { return cast(V) N; }
k :: inline (a: Vec($T, $N)) -> int #modify { return N > 0; } { return N; }
main :: () {
    a: [3] int;
    print("% % % %\n", f(Vec(u8, 16).{}), g(a), h(s64, Vec(u8, 5).{}), k(Vec(u8, 7).{}));
}
