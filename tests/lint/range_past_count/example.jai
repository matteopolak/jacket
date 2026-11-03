#import "Basic";

main :: () {
    names := string.["a", "b", "c"];
    for i: 0..names.count {
        print("%\n", names[i]);
    }
}
