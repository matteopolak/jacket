S :: struct { p: *S; x: int; }
f :: () -> S { s: S; s.p = *s; return s; }
V :: #run f();
main :: () { v := V; y := v.p.x; }
