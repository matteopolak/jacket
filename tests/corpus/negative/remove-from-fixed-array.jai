// A fixed array cannot shrink, so `remove` over one is an error.
#import "Basic";
main :: () {
    a: [3] int;
    for a remove it;
}
