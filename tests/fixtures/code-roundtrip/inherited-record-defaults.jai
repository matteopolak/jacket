#import "Basic";

Base :: struct {
    kind: enum {
        EMPTY;
        READY;
    }
    untouched: s64 = 71;
}

Derived :: struct {
    #as using base: Base = .{ kind = .READY };
    name: string;
}

verify :: () -> bool {
    local: Derived;
    named := Derived.{ name = "derived" };
    allocated := New(Derived);
    result := local.kind == .READY && named.kind == .READY && allocated.kind == .READY;
    result = result && local.untouched == 71 && named.untouched == 71 && allocated.untouched == 71;
    free(allocated);

    return result;
}

#assert verify();

main :: () -> int {
    if !verify() return 1;
    return 0;
}
