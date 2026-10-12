#import "Basic";
Vec :: struct (T: Type, N: int) { lanes: [N] T; }
f :: (a: Vec($T, $N)) -> int #modify { if N > 4 return false, "too many lanes"; return true; } { return N; }
main :: () {
    print("%\n", f(Vec(u8, 16).{}));
}
