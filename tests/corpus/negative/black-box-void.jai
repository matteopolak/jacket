// A void value has nothing for black_box to hide from the optimizer.
#import "Extensions/Testing";

main :: () {
    v: void;
    black_box(v);
}
