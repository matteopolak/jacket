S :: struct { a: int; b: float; }
main :: () { s: [4] S; for * s { it.a = 1; } }
