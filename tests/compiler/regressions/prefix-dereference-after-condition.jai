// A `(.*)` that follows a parenthesized condition is a prefix dereference starting the branch,
// not an argument list calling the condition.
// rules: ptr.22
#import "Basic";

main :: () {
    x := 5;
    p := *x;
    found := true;

    if (found && (p != null))(.*) p = 9;

    assert(x == 9);

    if found && (p != null)(.*) p = 11;

    assert(x == 11);

    if (x == 11)(.*) p = 13;
    else (.*) p = 1;

    assert(x == 13);
    print("ok\n");
}
