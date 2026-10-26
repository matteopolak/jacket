#import "Basic";
#import "Extensions/Variant";

// A case that holds the variant itself has no size: only a pointer may close the loop.
Tree :: Variant(Leaf, Branch);

Leaf :: struct {
    value: s64;
}

Branch :: struct {
    inner: Tree;
}

main :: () {
    tree: Tree;
}
