#import "Basic";
f :: () -> string { return copy_string("abc"); }
S :: #run f();
main :: () { print("%\n", S); }
