// A `#c_call` procedure has no context, so it cannot call `print`.
#import "Basic";
cb :: (x: int) -> int #c_call { print("%\n", x); return x; }
main :: () { cb(1); }
