set :: (p: *int) { p.* += 9; }
main :: () -> int { x := 6; set(*x); q: *int = null; if q != null return 1; return x; }
