f :: (x: int) #c_call { y := x; }
main :: () { f(1); p := cast(*void) f; }
