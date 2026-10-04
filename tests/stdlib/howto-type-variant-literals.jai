#import "Basic";
#import "Math";

// String literals convert to `#type,isa` / `#type,distinct` string variants, array literals
// to distinct arrays, and base-type operators on isa variants give the variant back.

Filename :: #type,isa string;
Grid3 :: #type,distinct [3] s32;
Pos :: #type,isa Vector3;

length :: (name: Filename) -> s64 { return name.count; }

main :: () {
    assert(length("Hello") == 5);

    g: Grid3 = .[1, 4, 9];
    assert(g[2] == 9);

    a: Pos = .{1, 2, 3};
    b: Pos = .{4, 5, 6};
    c := a + b;
    assert(type_of(c) == Pos);
    assert(c.y == 7);
}
