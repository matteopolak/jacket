#import "Basic";

set :: (a: [3] int) {
    a[0] = 5;
}

main :: () {
    set(.[1, 2, 3]);
}
