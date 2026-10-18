// An untyped array literal gives `$N` its count but cannot give `$T` its element type.
#import "Basic";

g :: (xs: [$N] $T) { print("%\n", N); }

main :: () {
    g(.[1, 2, 3]);
}
