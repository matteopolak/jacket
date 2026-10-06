g :: () -> int { return 5; }
f :: () -> () -> int { return g; }
P :: #run f();
main :: () { x := P(); }
