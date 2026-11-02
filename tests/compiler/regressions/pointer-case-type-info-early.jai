// A `#insert` that reads the type info of a pointer to a struct not laid out yet sees the
// struct's name, even after an earlier `#run` has already run.
#import "Basic";
#import "Extensions/Variant";

Earlier :: Variant(s64, string);

Branch_Node :: struct {
    children: int;
}

Mixed :: Variant(*Branch_Node, s64);

main :: () {
    e: Earlier;
    if Mixed.case_names[0] != "branch_node" exit(1);
    print("ok\n");
}
