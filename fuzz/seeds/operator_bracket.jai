V :: struct { x: int; }
operator [] :: (a: V, i: int) -> int { return a[i]; }
main :: () { a: V; b := a[1]; }
