#import "Basic";
main :: () { p := alloc(-100); q := realloc(null, -1, 0); free(p); free(cast(*void) 1); }
