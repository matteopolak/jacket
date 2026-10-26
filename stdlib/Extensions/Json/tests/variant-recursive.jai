// A variant that holds pointers to itself, encoded and decoded as nested JSON.
//
//     jaic stdlib/Extensions/Json/tests/variant-recursive.jai -interp

#import "Basic";
#import "Extensions/Json";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Tree :: Variant(Leaf, Branch);

Leaf :: struct {
    value: s64;
}

Branch :: struct {
    children: [..] *Tree;
}

leaf :: (value: s64) -> *Tree {
    result := New(Tree);

    set(result, Leaf.{ value });

    return result;
}

round_trip :: () {
    root: Tree;
    branch: Branch;

    array_add(*branch.children, leaf(1));

    inner := New(Tree);
    nested: Branch;

    array_add(*nested.children, leaf(2));
    set(inner, nested);
    array_add(*branch.children, inner);
    set(*root, branch);

    text := to_json(root,, temp);

    check(text == "{\"Branch\":{\"children\":[{\"Leaf\":{\"value\":1}},{\"Branch\":{\"children\":[{\"Leaf\":{\"value\":2}}]}}]}}", tprint("encoded as %", text));

    decoded, error := from_json(Tree, text);

    check(error.kind == .NONE, tprint("decoding failed: %", error.message));
    check(to_json(decoded,, temp) == text, "the decoded tree encodes the same again");

    free_json(*decoded);
}

main :: () {
    round_trip();

    if failures print("% check(s) failed\n", failures);
    else print("ok\n");
}
