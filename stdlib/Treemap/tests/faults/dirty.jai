T :: #import "Treemap";

main :: () {
    tree: T.Treemap;
    T.add_node(*tree, "root", null);
    display := T.init_treemap_display(*tree);
    T.recompute_if_dirty(*display, .{ 10, 10 });
}
