#import "Basic";

set :: (a: [] int) {
    a.count = 1;
}

main :: () {
    set(.[1, 2]);
}
