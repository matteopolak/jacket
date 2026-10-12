#import "Basic";
S :: struct { a: [] u32; b: [8] u8; n: int; }
make :: (n: int) -> S { s: S; s.a = NewArray(3, u32); s.a[1] = 7; s.n = n; return s; }
G :: #run make(4);
get_g :: () -> *S { return *G; }
get_l :: () -> *S { C :: #run make(6); return *C; }
get_p :: ($n: int) -> *S { C :: #run make(n); p := *C; return p; }
main :: () {
    print("% %\n", get_g().n, get_g().a);
    print("% %\n", get_l().n, get_l().a);
    print("% %\n", get_p(5).n, get_p(5).a);
    print("%\n", get_p(5) == get_p(5));
}
