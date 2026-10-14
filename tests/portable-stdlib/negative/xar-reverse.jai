#import "Basic";
X :: #import "Xar";

main :: () {
    array: X.Xar(s64, 2, 3);
    for < value: array assert(value >= 0);
}
