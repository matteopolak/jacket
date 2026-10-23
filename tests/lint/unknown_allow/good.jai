#import "Basic";

// jailint: allow(unused_variable)
a :: () {
    x := 1;
}

b :: () {
    y := 2; // jailint: allow(unused_variable, style, all)
}

// The name `no_such_rule` in prose, or a different marker, is not an allow list.
// jailint: allowing nothing
// lint: allow(no_such_rule)

main :: () {
    a();
    b();
    print("hello\n");
}
