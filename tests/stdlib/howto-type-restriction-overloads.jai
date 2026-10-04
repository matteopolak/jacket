#import "Basic";

// `$T/Base` only matches types that are `Base` or have it as an `#as` base, so a second
// overload restricted to an unrelated struct gets the other calls.

Entity :: struct { id: s32; name: string; }
Tree :: struct { #as using base: Entity; seeds: s32; }
Strange :: struct { x, y: float64; }

describe :: (x: $T/Entity) -> string { return x.name; }
describe :: (x: $T/Strange) -> string { return "strange"; }

main :: () {
    tree: Tree;
    tree.name = "oak";
    strange: Strange;
    assert(describe(tree) == "oak");
    assert(describe(strange) == "strange");
}
