V :: struct { x: int; }
operator + :: (a: V, b: V) -> V { return a + b; }
main :: () { a: V; b := a + a; }
