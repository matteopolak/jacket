twice :: (x: $T) -> T { return x+x; }
main :: () -> int { return twice(9); }
