Pair :: struct { a:s32=3; b:s32=5; }
main :: () -> int { p:=initializer_of(Pair); return size_of(type_of(p))+p.a+p.b; }
