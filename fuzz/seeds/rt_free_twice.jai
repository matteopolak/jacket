#import "Basic";
main :: () { p := alloc(10); free(p); free(p); }
