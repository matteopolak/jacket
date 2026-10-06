#import "Basic";

main :: () {
    xs := int.[1, 2, 3];
    for i: 10..0 print("%\n", i);
    for i: xs.count - 1..0 print("%\n", xs[i]);
}
