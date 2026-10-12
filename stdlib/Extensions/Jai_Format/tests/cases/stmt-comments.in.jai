run :: () {
    // first comment stays at the top
    a();
    // gets a blank above
    b();
    // two comment lines
    // belong together
    c(); // trailing stays
    /* a block comment */
    d();
    if x {
        // after a brace: no blank
        e();
        // before the closing brace
    }
    call(a,
         // inside an expression: untouched
         b);

    // already separated
    f();
    if y {
        g();
    }
    // after a block
    // else-less
    h();
    if k == {
        case 1;
            // after a case label
            i();
            // between statements
            j();
        // before a case
        case 2;
            k();
    }
}

Thing :: struct {
    a: int;
    // struct members are not code
    b: int;
}
// file level is not code
x :: 1;
