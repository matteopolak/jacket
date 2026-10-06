// Checks the claims of docs/language/declarations-and-constants.md, one group per rule.
// rules: decl.1 decl.3 decl.4 decl.5 decl.6 decl.7 decl.8 decl.9 decl.11 decl.13 decl.15 decl.16 decl.17 decl.18 decl.20
#import "Basic";

// decl.1: file-level names may be used before their declaration.
EARLY :: LATE * 2;
LATE :: 21;

// decl.3: the example's forms.
LIMIT :: 4;
SCALE: float : 2.5;
Word :: u16;
answer: Word : 42;
counter := 10;
table: [LIMIT] int;
squares := #run make_squares();

make_squares :: () -> [4] int {
    result: [4] int;
    for 0..3 result[it] = it * it;
    return result;
}

Rec :: struct {
    TRUE: s32 : 1;
    LIM :: 3;
    Count :: 2;
    read :: (r: Rec) -> int {
        return r.v * LIM;
    }
    v: s32 = 9 @Hidden;
    w: s32 = 5;
}

#assert size_of(Rec) == 8 "Rec must be 8 bytes";
#assert(size_of(Rec) == 8, "Rec must be 8 bytes");
#assert LIMIT == 4;

pair :: () -> int, string {
    return 3, "three";
}
triple :: () -> int, int, int {
    return 1, 2, 3;
}

OUTER_CONST :: 11;
outer_global := 5;

main :: () {
    // decl.1: block constants are visible before their declaration.
    early_use := HOISTED + 1;
    HOISTED :: 9;
    assert(early_use == 10, "hoisted block constant");
    assert(EARLY == 42, "file-level order independence");

    // decl.3
    assert(SCALE == 2.5, "typed constant");
    #assert type_of(answer) == Word;
    assert(answer == 42, "alias-typed constant");
    counter += 1;
    assert(counter == 11, "mutable global");
    assert(table.count == 4, "constant array size");
    assert(squares[0] == 0 && squares[3] == 9, "#run global");

    // decl.4: local types, constants and procedures refer to each other regardless of order.
    Count :: Later + 1;
    Later :: 2;
    Local :: struct {
        values: [Count] int;
    }
    local_total :: () -> int {
        return helper() + Count;
    }
    helper :: () -> int {
        return 10;
    }
    l: Local;
    assert(l.values.count == 3, "local struct uses a later constant");
    assert(local_total() == 13, "local procedure calls a later one");

    // decl.5: struct constants and procedures are namespace members.
    r: Rec;
    assert(Rec.Count == 2 && Rec.LIM == 3, "struct constants");
    assert(Rec.read(r) == 27, "struct procedure");

    // decl.6: a typed constant in a struct is not a field.
    assert(Rec.TRUE == 1, "constant member");
    assert(type_info(Rec).members.count == 2, "only v and w are fields");
    assert(size_of(Rec) == 8, "constants take no space");

    // decl.7: `---` declares without initialising.
    u: int = ---;
    u = 3;
    assert(u == 3, "uninitialised declaration");

    // decl.8: several declarations at once.
    a, b := 1, 2.5;
    #assert type_of(a) == int;
    #assert type_of(b) == float;
    assert(a == 1 && b == 2.5, "multi-declaration");

    // decl.9: field defaults apply on declaration.
    assert(r.v == 9 && r.w == 5, "field defaults");

    // decl.11: nested procedures see enclosing constants, types and globals.
    LOCAL_CONST :: 4;
    Inner :: struct {
        n: int;
    }
    nested :: () -> int {
        i: Inner;
        i.n = LOCAL_CONST;
        return i.n + OUTER_CONST + outer_global;
    }
    assert(nested() == 20, "nested procedure");

    // decl.15: `a, b :: f();` binds every value.
    N, S :: pair();
    assert(N == 3 && S == "three", "multi-value constant");

    // decl.16: fewer names than values take the leading values.
    P, Q :: triple();
    assert(P == 1 && Q == 2, "leading values");

    // decl.17: `#run f()` keeps all values.
    // (The variable form `x, y, z := #run triple();` is pending: jaic repeats the first value.)
    X, Y, Z :: #run triple();
    assert(X == 1 && Y == 2 && Z == 3, "#run keeps every value");

    // decl.18: both #assert forms run at compile time (the file-level ones above held).
    #assert Count == 3 "local constant";
    #assert(Later == 2, "parenthesized form");

    // decl.20: notes after a field.
    member := type_info(Rec).members[0];
    assert(member.name == "v" && member.notes.count == 1 && member.notes[0] == "Hidden", "field note");

    print("ok\n");
}
