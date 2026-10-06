// rules: union.12 union.13 union.14 flow.26
// A tagged union's members may be selected by integer or type constants, not only enum
// members (sjorsdonkers/toml-jai): `4,, a: u8;`, `-12,, c: s16;`, `u16,, a: s8;`. The
// type info lists each (constant, member index) pair.
#import "Basic";

Small :: union kind: s8 = 4 {
    4,, a: u8;
    b: string = ---;
    -12,, c: s16 = ---;
}

ByType :: union kind: Type = u16 {
    u16,, a: s8;
    s32,, b: u16;
}

main :: () {
    s := Small.{ kind = 4, a = 42 };
    assert(s.kind == 4 && s.a == 42);

    info := type_info(Small);
    assert(info.tagged_union_bindings.count == 2);
    first := info.tagged_union_bindings[0];
    second := info.tagged_union_bindings[1];
    assert(first.constant_value == 4 && info.members[first.member_index].name == "a");
    assert(cast(s64) second.constant_value == -12 && info.members[second.member_index].name == "c");

    t := ByType.{ kind = s32, b = 300 };
    assert(t.kind == s32 && t.b == 300);
    tinfo := type_info(ByType);
    assert(tinfo.tagged_union_bindings.count == 2);
    for tinfo.tagged_union_bindings {
        tag := (cast(*Type) *it.constant_value).*;
        name := tinfo.members[it.member_index].name;
        assert((tag == u16 && name == "a") || (tag == s32 && name == "b"), "% -> %", tag, name);
    }

    // `ifx` as a statement whose branches assign (toml-jai's date validation).
    leap := true;
    max_day: u8;
    ifx leap then max_day = 29 else max_day = 28;
    assert(max_day == 29);
    leap = false;
    ifx leap then max_day = 29 else max_day = 28;
    assert(max_day == 28);

    // `ok=, p.* = f();` assigns to both targets (toml-jai's parser).
    pair :: () -> bool, int {
        return true, 7;
    }
    ok := false;
    slot: int;
    p := *slot;
    ok =, p.* = pair();
    assert(ok && slot == 7);
    print("ok\n");
}
