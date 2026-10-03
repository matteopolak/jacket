pair :: () -> (int, int) { return 8, 11; }
main :: () -> int { a, b := pair(); return a+b; }
