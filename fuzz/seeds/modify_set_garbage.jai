f :: (x: $T) #modify { T = cast(Type) 12345; return true; } {}
main :: () { f(1); }
