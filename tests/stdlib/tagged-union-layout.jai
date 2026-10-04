// A tagged union (`union kind: T { .A ,, a: X; }`) is a struct: the tag field, then an
// anonymous union of the members, all reachable directly.
#import "Basic";
Kind :: enum u8 { INT; TEXT; PAIR; }
Value :: union kind: Kind {
    .INT  ,, int_value: s64;
    .TEXT ,, text: string;
    .PAIR ,, pair: struct { a: s32; b: s32; };
}
make :: (n: s64) -> Value { return .{ kind = .INT, int_value = n }; }
main :: () {
    v := make(42);
    t: Value = .{ kind = .TEXT, text = "hi" };
    p: Value; p.kind = .PAIR; p.pair.b = 9;
    assert(v.kind == .INT && v.int_value == 42 && t.text == "hi" && p.pair.b == 9);
    assert(size_of(Value) == 24 && type_info(Value).type == .STRUCT);

}
