// A tagged union's tag may have a default: `union tag: T = .B {` or `union tag := T.B {` (the tag
// type comes from the default). Seen in sjorsdonkers/toml-jai, which real Jai compiles.
#import "Basic";

Kind :: enum u8 {
    NUMBER;
    TEXT;
    LIST;
}

Typed :: union kind: Kind = .TEXT {
    .NUMBER,, number: s64;
    .TEXT,, text: string;
}

Inferred :: union kind := Kind.LIST {
    .NUMBER,, number: s64;
    .LIST,, items: [..] s64;
}

Holder :: struct {
    name: string;
    union tag := Kind.NUMBER {
        .NUMBER,, n: s64;
        .TEXT,, t: string;
    };
}

main :: () {
    a: Typed;
    assert(a.kind == .TEXT);
    assert(type_of(a.kind) == Kind);

    b: Inferred;
    assert(b.kind == .LIST);
    assert(type_of(b.kind) == Kind);

    c := Inferred.{ kind = .NUMBER, number = 7 };
    assert(c.kind == .NUMBER && c.number == 7);

    h: Holder;
    assert(h.tag == .NUMBER);
    h.tag = .TEXT;
    h.t = "x";
    assert(h.t == "x");
}
