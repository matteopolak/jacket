// A constant array literal returned as a view outlives the procedure; an empty one has null data.
// rules: ptr.16 ptr.17
#import "Basic";

words :: () -> bool, [] string {
    return true, .["The", "number"];
}
numbers :: () -> [] int {
    return .[1, 2, 3];
}
EMPTY :: string.[];

main :: () {
    ok, w := words();
    n := numbers();
    filler: [64] int;
    for *filler it.* = -1; // reuse stack space
    if w.count != 2 || w[0] != "The" || w[1] != "number" {
        print("FAIL words %\n", w);
        exit(1);
    }
    if n.count != 3 || n[2] != 3 {
        print("FAIL numbers %\n", n);
        exit(1);
    }
    e := string.[];
    if e.data != null || EMPTY.data != null {
        print("FAIL empty data\n");
        exit(1);
    }
    print("ok\n");
}
