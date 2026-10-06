f :: (n: int) -> int { a: [100000] int; a[0] = n; return f(n + 1) + a[0]; }
main :: () { x := f(0); }
