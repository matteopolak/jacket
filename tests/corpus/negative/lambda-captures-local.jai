// A lambda cannot read a local of the procedure around it.
#import "Basic";
main :: () {
    k := 3;
    add := (x: int) => x + k;
    print("%\n", add(1));
}
