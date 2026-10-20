// `continue` can only name an enclosing loop.
#import "Basic";
main :: () {
    for 1..3 { continue nowhere; }
}
