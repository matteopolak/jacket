// Where an expression is expected, `{...}` without a leading dot is a struct literal whose type
// comes from the context, like `.{...}`: call arguments, typed declarations, assignments, return
// values and fields of other literals. A `{` that starts with a statement or holds `;` is a block.
// rules: struct.17
#import "Basic";

fail :: (code: int, what: string) {
    print("FAIL %: %\n", code, what);
    exit(xx code);
}

Kind :: enum { GENERAL; COMPUTE; }

Desc :: struct {
    kind: Kind;
    count: int;
}

Extent :: struct {
    width, height: int;
    desc: Desc;
}

describe :: (d: Desc) -> int {
    return cast(int) d.kind * 100 + d.count;
}

make :: (w: int) -> Extent {
    return {w, w * 2, {.COMPUTE, 7}};
}

main :: () {
    if describe({.GENERAL, 1}) != 1 fail(1, "positional argument");
    if describe({count = 3, kind = .COMPUTE}) != 103 fail(2, "named argument");

    e: Extent = {1280, 720, {.COMPUTE, 2}};
    if e.width != 1280 || e.height != 720 || e.desc.count != 2 fail(3, "typed declaration");

    e = {};
    if e.width != 0 || e.desc.kind != .GENERAL fail(4, "empty literal");

    e = {width = 5};
    if e.width != 5 || e.height != 0 fail(5, "assignment");

    m := make(3);
    if m.height != 6 || m.desc.kind != .COMPUTE || m.desc.count != 7 fail(6, "return value");

    // A block in expression position still runs as a block.
    total := 0;
    for 1..3 {
        total += it;
    }
    if total != 6 fail(7, "block");

    print("ok\n");
}
