select :: (x: int) -> int { return x+2; }
select :: (x: bool) -> int { if x return 19; return 3; }
main :: () -> int { return select(true) + select(5); }
