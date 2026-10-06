f :: (n: int) -> int { return f(n + 1) + 1; }
main :: () { x := f(0); }
