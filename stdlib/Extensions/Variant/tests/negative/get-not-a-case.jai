#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string);
    value, ok := get(v, float64);
}
