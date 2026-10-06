#import "Basic";
f :: () -> [..] int { a: [..] int; array_add(*a, 1); return a; }
V :: #run f();
main :: () { print("%\n", V); }
