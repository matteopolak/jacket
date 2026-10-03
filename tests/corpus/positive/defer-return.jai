state := 2;
compute :: () -> int { defer state += 5; return state; }
main :: () -> int { captured := compute(); return captured + state; }
