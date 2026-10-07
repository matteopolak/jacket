// rules: ext.9
// Like float64, Long_Double takes an s64 only through an explicit cast.
#import "Extensions/Long_Double";
main :: () {
    n: s64 = 7;
    x: Long_Double = n;
}
