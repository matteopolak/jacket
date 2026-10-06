// rules: ext.9
// Like float64, Long_Double takes an s64 only through an explicit cast.
#import "Jaic_Extensions";
main :: () {
    n: s64 = 7;
    x: Long_Double = n;
}
