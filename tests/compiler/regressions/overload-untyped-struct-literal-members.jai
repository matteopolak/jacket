// An untyped `.{...}` argument does not pick a `string` parameter (whose built-in members are
// `data` and `count`) when its values cannot fill those members: here an empty `.[]` first.
#import "Basic";

Part :: struct {
    names: [] string;
    text: string;
}

make :: (a: string, b: string) -> int {
    return make(.{ .[], a }, .{ .[], b });
}

make :: (a: Part, b: Part) -> int {
    return a.names.count + b.names.count + a.text.count + b.text.count;
}

make :: (a: Part, b: string) -> int {
    return make(a, .{ .[], b });
}

main :: () {
    assert(make("ab", "c") == 3);
    assert(make(.{ .[], "x" }, "yz") == 3);
}
