// A mixed declare-and-assign list may assign to members, array elements and dereferences:
// `ok:, t.str = f();` declares `ok` and stores the second result in `t.str`.
// rules: decl.22
#import "Basic";
#import "String";

fail :: (code: int, what: string) {
    print("FAIL %: %\n", code, what);
    exit(xx code);
}

Token :: struct {
    str: string;
    pos: [2] int;
}

split :: (s: string) -> bool, string, int {
    return true, slice(s, 1, s.count - 1), s.count;
}

measure :: (s: string) -> int, string {
    return s.count, slice(s, 1, s.count - 1);
}

main :: () {
    t: Token;
    ok:, t.str = split("xhello");
    if !ok || t.str != "hello" fail(1, "member target");

    found:, t.str, t.pos[1] = split("abc");
    if !found || t.str != "bc" || t.pos[1] != 3 fail(2, "member and index targets");

    count: int;
    p := *count;
    tail: string;
    good:, tail, p.* = split("four");
    if !good || tail != "our" || count != 4 fail(3, "dereference target");

    tokens: [3] Token;
    i := 2;
    done:, tokens[i].str = split("-x-");
    if !done || tokens[2].str != "x-" fail(4, "indexed member target");

    // `=,` marks the existing places when the rest are declared with `:=`.
    t.pos[0]=, rest := measure("zz");
    if t.pos[0] != 2 || rest != "z" fail(5, "place marked with `=,`");

    print("ok\n");
}
