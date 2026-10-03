ticks := 0;
pair :: () -> (first:int, second:int) { ticks += 1; return second=2, first=1; }
main :: () -> int { a,b := pair(); a,b = b,a; return a*10+b+ticks; }
