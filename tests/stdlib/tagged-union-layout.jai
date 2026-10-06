// A tagged union (`union kind: T { .A ,, a: X; }`) is a struct: the tag field, then an
// anonymous union of the members, all reachable directly.
#import "Basic";
Kind :: enum u8 {
    INT;
    TEXT;
    PAIR;
}
Value :: union kind: Kind {
    .INT,, int_value: s64;
    .TEXT,, text: string;
    .PAIR,, pair: struct {
        a: s32;
        b: s32;
    };
}
make :: (n: s64) -> Value {
    return .{ kind = .INT, int_value = n };
}
main :: () {
    v := make(42);
    t: Value = .{ kind = .TEXT, text = "hi" };
    p: Value;
    p.kind = .PAIR;
    p.pair.b = 9;
    assert(v.kind == .INT && v.int_value == 42 && t.text == "hi" && p.pair.b == 9);
    assert(size_of(Value) == 24 && type_info(Value).type == .STRUCT);
    flags := type_info(Value).textual_flags;
    assert(flags & .UNION && flags & .UNION_IS_TAGGED);
    // Type info lists the tag, then each variant, with the tag values selecting them.
    info := type_info(Value);
    assert(info.members.count == 4 && info.members[0].name == "kind" && info.members[2].name == "text");
    assert(info.members[2].offset_in_bytes == 8);
    assert(info.tagged_union_bindings.count == 3);
    b := info.tagged_union_bindings[1];
    assert(b.constant_value == xx Kind.TEXT && b.member_index == 2);

}
