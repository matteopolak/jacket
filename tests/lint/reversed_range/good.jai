#import "Basic";

main :: () {
    xs := int.[1, 2, 3];
    for < i: 0..10 print("%\n", i);
    for < i: 0..xs.count - 1 print("%\n", xs[i]);
    for i: 0..0 print("%\n", i);
}
