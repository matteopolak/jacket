// rules: ext.8
// `%` is not an operator on Long_Double (whether it is wide or float64 itself).
#import "Extensions/Long_Double";
main :: () {
    x: Long_Double = 7;
    y := x % 2;
}
