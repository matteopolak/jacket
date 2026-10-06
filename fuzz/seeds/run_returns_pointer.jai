f :: () -> *int { x := 5; return *x; }
P :: #run f();
main :: () { y := P.*; }
