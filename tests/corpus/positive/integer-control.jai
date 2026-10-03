square :: (x: int) -> int { return x*x; }
main :: () -> int { sum := 0; for n: 1..4 { sum += square(n); } return sum; }
