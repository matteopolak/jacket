Pair :: struct { a:s32=3; b:s32=5; }
main :: () -> int { p: Pair = ---; initializer_of(Pair)(*p); return size_of(type_of(p))+p.a+p.b; }
