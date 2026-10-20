// `remove` does not apply to a `while` loop.
#import "Basic";
main :: () {
    a: [..] int;
    while a.count > 0 { remove; }
}
